#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)] // For bindings

use std::{
    collections::HashMap,
    fs::File,
    mem::size_of,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc, RwLock,
    },
    thread::{self, JoinHandle},
};

use clap::Parser;
use log::{debug, error, info};
use perf::{
    perf_event_sample_format_PERF_SAMPLE_IDENTIFIER,
    perf_sys::{
        perf_event_attr, perf_event_header, perf_event_sample_format_PERF_SAMPLE_ADDR,
        perf_event_sample_format_PERF_SAMPLE_TID, perf_event_sample_format_PERF_SAMPLE_TIME,
        perf_type_id_PERF_TYPE_RAW,
    },
    PerfError,
};

use crate::perf::{perf_event_sample_format_PERF_SAMPLE_IP, PerfEvent};

mod numa_sys {
    include!(concat!(env!("OUT_DIR"), "/numa-sys.rs"));
}

mod perf;

type PageT = u64;
type CostT = u64;
type NodeT = u32;

const SAMPLE_FREQ: u64 = 4000;
const PAGE_SIZE: u64 = 4096;

// Should be same for TGL and SKX
const L3MISS: u64 = 0xd1 | (0x20 << 8);
const ALLSTORES: u64 = 0xd0 | (0x82 << 8);

const pol_flag_wait: u8 = 0;
const pol_flag_run: u8 = 1;
const pol_flag_stop: u8 = 2;

#[derive(Parser)]
struct Args {
    #[clap(short, long, default_value = "0")]
    pid: i32,
    #[clap(short, long, default_value = "-1")]
    cpu: i32,
    #[clap(short, long, default_value = "1000")]
    sample_period: u64,
}

#[derive(Default, Debug)]
#[repr(C)]
struct demo_record {
    hdr: perf_event_header,
    id: u64,
    ip: u64,
    tid: u64,
    time: u64,
    addr: u64,
}

trait Tracker {
    fn update(&mut self, addr: u64);
    fn execute(&self);
    fn debug_summary(&self);
    fn handle_sample(&mut self, sample: *const u8);
}

struct Policy {
    handle: Option<JoinHandle<()>>,
    tiers: Vec<Vec<(PageT, CostT)>>,
}

impl Policy {
    fn new() -> Self {
        Self {
            handle: None,
            tiers: vec![vec![], vec![]],
        }
    }

    fn start(
        &mut self,
        target_pid: i32,
        pol_flag: Arc<AtomicU8>,
        tracking: Arc<RwLock<HashMap<PageT, (CostT, NodeT)>>>,
    ) {
        if let None = self.handle {
            self.handle = Some(thread::spawn(move || loop {
                let tracking_clone = tracking.clone();
                match pol_flag.load(Ordering::Relaxed) {
                    pol_flag_wait => {
                        thread::park();
                    }
                    pol_flag_run => {
                        Self::execute(target_pid, tracking_clone);
                        pol_flag.store(pol_flag_wait, Ordering::Relaxed);
                    }
                    pol_flag_stop => {
                        break;
                    }
                    _ => {
                        error!("Invalid policy flag.");
                        break;
                    }
                }
            }))
        }
    }

    fn notify(&self) {
        if let Some(handle) = self.handle.as_ref() {
            handle.thread().unpark();
        }
    }

    fn join(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.thread().unpark();
            handle.join().unwrap();
        }
    }

    fn execute(target_pid: i32, tracking: Arc<RwLock<HashMap<PageT, (CostT, NodeT)>>>) {
        // Update tiers lists
        let entries = tracking.read().unwrap();
        let fast_tier_len = entries.iter().filter(|(_, (_, node))| *node == 0).count();
        let slow_tier_len = entries.iter().filter(|(_, (_, node))| *node == 1).count();
        // * Threshold conditions check*
        // Maintain ratio of 4:1 for tier 0 to tier 1
        // If fast tier is 4 times larger than slow tier, migrate pages from fast to slow
        // If slow tier is more than 1/4th of fast tier, migrate pages from slow to fast
        let (target_node, ratio) = match (fast_tier_len, slow_tier_len) {
            (fast_len, slow_len) if fast_len > 4 * slow_len => (1, fast_len / (4 * (1 + slow_len))),
            (fast_len, slow_len) if slow_len > fast_len / 2 => (0, slow_len / ((1 + fast_len) / 2)),
            _ => return,
        };
        let mut candidates = entries
            .iter()
            .filter(|(_, (_, node))| *node == 1 - target_node)
            .map(|(addr, _)| *addr)
            .take(ratio)
            .collect::<Vec<_>>();
        drop(entries);
        let n = candidates.len();
        let nodes: Vec<i32> = vec![target_node as i32; n];
        let mut status = vec![-123; n];
        let ret = unsafe {
            numa_sys::move_pages(
                target_pid,
                n as u64,
                candidates.as_mut_ptr().cast(),
                nodes.as_ptr(),
                status.as_mut_ptr(),
                0,
            )
        };
        if ret < 0 {
            error!("Failed to move pages.");
        } else {
            let mut entries = tracking.write().unwrap();
            for (p, s) in candidates.iter().zip(status.iter()) {
                if *s < 0 {
                    error!("Failed to move page {:#x}.", p);
                } else {
                    if let Some((_, node)) = entries.get_mut(p) {
                        // Checked that status is non-negative
                        *node = *s as u32;
                    }
                }
            }
            if n > 0 {
                debug!(
                    "Moved {} pages from tier {} to tier {}.",
                    n,
                    1 - target_node,
                    target_node
                );
            }
        }
    }
}

struct PolTracker {
    pol_flag: Arc<AtomicU8>,
    pol_thread: Policy,
    pid: i32,
    inner: Arc<RwLock<HashMap<PageT, (CostT, NodeT)>>>,
}

impl PolTracker {
    fn new(pid: i32) -> Self {
        Self {
            pol_flag: Arc::new(AtomicU8::new(0)),
            pol_thread: Policy::new(),
            pid,
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn start_policy(&mut self) {
        self.pol_thread
            .start(self.pid, self.pol_flag.clone(), self.inner.clone());
        info!("Started policy thread.");
    }
}

impl Drop for PolTracker {
    fn drop(&mut self) {
        self.pol_flag.store(pol_flag_stop, Ordering::Relaxed);
        self.pol_thread.join();
        debug!("Stopped policy thread.");
    }
}

impl Tracker for PolTracker {
    // Update cost associated with a page. Assumes addr is page aligned.
    fn update(&mut self, page: PageT) {
        //let entry = self.inner.entry(page).or_insert(0);
        if let Ok(mut inner) = self.inner.write() {
            let entry = inner.entry(page).or_insert((0, 0));
            (*entry).0 += 1;
        } else {
            error!("Failed to update page cost.");
        }
    }

    fn execute(&self) {
        self.pol_flag.store(pol_flag_run, Ordering::Relaxed);
        self.pol_thread.notify();
    }

    fn debug_summary(&self) {
        let entries = self.inner.read().unwrap();
        info!("Total pages: {}", entries.len());
        let mut entries = entries.iter().collect::<Vec<_>>();
        let sample_n = entries.len().min(10);
        entries.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap());
        for (addr, cost) in entries.iter().rev().take(sample_n) {
            info!("{:#x}: {:.2}, node: {}", addr, cost.0, cost.1);
        }

        for (addr, cost) in entries.iter().take(sample_n) {
            info!("{:#x}: {:.2}, node: {}", addr, cost.0, cost.1);
        }
    }
    // Caller must ensure ptr is valid.
    fn handle_sample(&mut self, sample: *const u8) {
        let sample: *const demo_record = sample.cast();

        let va = unsafe { (*sample).addr & !(PAGE_SIZE - 1) };
        self.update(va);

        //unsafe {
        //    debug!(
        //        "[ID: {}] {:#02x},{},{},{:#02x}",
        //        (*sample).id,
        //        (*sample).ip,
        //        (*sample).tid,
        //        (*sample).time,
        //        (*sample).addr,
        //    );
        //}
    }
}

fn build_mem_event(
    args: &Args,
    event: u64,
    disabled: bool,
    group: Option<&File>,
) -> Result<PerfEvent, PerfError> {
    let mut attr = perf_event_attr {
        type_: perf_type_id_PERF_TYPE_RAW,
        config: event,
        ..Default::default()
    };
    attr.set_sample_period(args.sample_period);
    attr.sample_type = perf_event_sample_format_PERF_SAMPLE_IDENTIFIER
        | perf_event_sample_format_PERF_SAMPLE_IP
        | perf_event_sample_format_PERF_SAMPLE_TID
        | perf_event_sample_format_PERF_SAMPLE_TIME
        | perf_event_sample_format_PERF_SAMPLE_ADDR;
    attr.__bindgen_anon_2.wakeup_events = (args.sample_period / 4) as u32;
    //attr.set_watermark(1); // Set this for wakeup watermark
    if disabled {
        attr.set_disabled(1);
    }
    attr.set_exclude_kernel(1);
    attr.set_exclude_hv(1);
    attr.set_exclude_callchain_user(1);
    attr.set_exclude_callchain_kernel(1);
    attr.set_precise_ip(2);
    PerfEvent::new(attr, args.pid, args.cpu, group, 0)
}

fn main() {
    env_logger::init();
    let args = Args::parse();
    let mut tracker = PolTracker::new(args.pid);
    tracker.start_policy();
    let mem_read = build_mem_event(&args, L3MISS, true, None).unwrap();
    let _mem_store = build_mem_event(&args, ALLSTORES, false, Some(mem_read.get_fd()));
    mem_read.reset().unwrap();
    mem_read.enable().unwrap();
    mem_read
        .sample_loop(size_of::<demo_record>(), &mut tracker)
        .unwrap();
    tracker.debug_summary();
}

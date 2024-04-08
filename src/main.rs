#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)] // For bindings

use std::{collections::HashMap, fs::File, mem::size_of};

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

const SAMPLE_FREQ: u64 = 4000;
const PAGE_SIZE: u64 = 4096;
const TOP_N: usize = 10;
const HEAT: f64 = 100.0;

// Should be same for TGL and SKX
const L3MISS: u64 = 0xd1 | (0x20 << 8);
const ALLSTORES: u64 = 0xd0 | (0x82 << 8);

#[derive(Parser)]
struct Args {
    #[clap(short, long, default_value = "0")]
    pid: i32,
    #[clap(short, long, default_value = "-1")]
    cpu: i32,
    #[clap(short, long, default_value = "10000")]
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

trait PolicyTracker {
    fn update(&mut self, addr: u64);
    fn execute(&mut self);
    fn debug_summary(&self);
}

#[derive(Default, Debug)]
struct Access {
    cost: f64,
    tier: usize,
    enqueued: bool,
}

impl Access {
    fn new(cost: f64, tier: usize) -> Self {
        Self {
            cost,
            tier,
            enqueued: false,
        }
    }
}

impl PartialOrd for Access {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.cost.partial_cmp(&other.cost)
    }
}

impl PartialEq for Access {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

#[derive(Default)]
struct Tracker {
    pid: i32,
    inner: HashMap<u64, Access>,
    tiers: [Vec<(u64, f64)>; 2],
}

impl Tracker {
    fn new(pid: i32) -> Self {
        Self {
            pid,
            inner: HashMap::new(),
            tiers: [Vec::new(), Vec::new()],
        }
    }
}

const ALPHA: f64 = 0.2;

impl PolicyTracker for Tracker {
    fn update(&mut self, addr: u64) {
        let entry = self.inner.entry(addr).or_insert(Access::new(0.0, 0));
        entry.cost += 1.0; //(1.0 - ALPHA) * *entry + ALPHA * 1.0;
        let m = self.tiers[entry.tier].last().unwrap_or(&(0, 1.0));
        if entry.cost > m.1 && !entry.enqueued {
            self.tiers[entry.tier].push((addr, entry.cost));
            entry.enqueued = true;
        }
    }

    fn execute(&mut self) {
        for (idx, entries) in self.tiers.iter_mut().enumerate() {
            entries.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            let n = entries.len();
            if n > TOP_N && entries.last().unwrap_or(&(0, 0.0)).1 > HEAT {
                let mut pages = entries
                    .drain(0..n - 1)
                    .into_iter()
                    .map(|(addr, _)| addr as *const u64)
                    .collect::<Vec<_>>();
                let nodes = (0..n - 1)
                    .map(|_| ((idx + 1) % 2) as i32)
                    .collect::<Vec<_>>();
                let mut status = vec![-123; n - 1];
                let ret = unsafe {
                    numa_sys::move_pages(
                        self.pid,
                        (n - 1) as u64,
                        pages.as_mut_ptr().cast(),
                        nodes.as_ptr(),
                        status.as_mut_ptr(),
                        0,
                    )
                };
                if ret < 0 {
                    error!("Failed to move pages.");
                } else {
                    for (p, s) in status.iter().enumerate() {
                        info!("Page {:#02x}, status: {}", p, s);
                    }
                }
            }
        }
    }

    fn debug_summary(&self) {
        for (tier, entries) in self.tiers.iter().enumerate() {
            info!("Tier: {}", tier);
            for (addr, access) in entries.iter().rev().take(entries.len().min(5)) {
                info!("{:#x}: {:.2}", addr, access);
            }
            for (addr, access) in entries.iter().take(entries.len().min(5)) {
                info!("{:#x}: {:.2}", addr, access);
            }
        }
        info!("-------------------");
        let mut entries: Vec<_> = self.inner.iter().collect();
        let sample_n = entries.len().min(10);
        entries.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap());
        for (addr, access) in entries.iter().rev().take(sample_n) {
            info!("{:#x}: {:.2}", addr, access.cost);
        }

        for (addr, access) in entries.iter().take(sample_n) {
            info!("{:#x}: {:.2}", addr, access.cost);
        }
    }
}

// Caller must ensure ptr is valid.
fn process_sample<T: PolicyTracker>(sample: *const u8, tracker: &mut T) {
    let sample: *const demo_record = sample.cast();

    let va = unsafe { (*sample).addr & !(PAGE_SIZE - 1) };
    tracker.update(va);

    unsafe {
        debug!(
            "[ID: {}] {:#02x},{},{},{:#02x}",
            (*sample).id,
            (*sample).ip,
            (*sample).tid,
            (*sample).time,
            (*sample).addr,
        );
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
    let mut tracker = Tracker::new(args.pid);
    let mem_read = build_mem_event(&args, L3MISS, true, None).unwrap();
    let _mem_store = build_mem_event(&args, ALLSTORES, false, Some(mem_read.get_fd()));
    mem_read.reset().unwrap();
    mem_read.enable().unwrap();
    mem_read
        .sample_loop(process_sample, size_of::<demo_record>(), &mut tracker)
        .unwrap();
    tracker.debug_summary();
}

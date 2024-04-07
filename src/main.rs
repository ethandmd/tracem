#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)] // For perf bindings

use std::{
    //collections::{BinaryHeap, HashMap},
    mem::size_of,
};

use clap::Parser;
use log::{debug, error};
use perf::{
    perf_event_sample_format_PERF_SAMPLE_IDENTIFIER,
    perf_sys::{
        perf_event_attr, perf_event_header, perf_event_sample_format_PERF_SAMPLE_ADDR,
        perf_event_sample_format_PERF_SAMPLE_TID, perf_event_sample_format_PERF_SAMPLE_TIME,
        perf_type_id_PERF_TYPE_RAW,
    },
};

use crate::perf::{perf_event_sample_format_PERF_SAMPLE_IP, PerfEvent};

mod perf;

const SAMPLE_FREQ: u64 = 4000;
const SAMPLE_PERIOD: u64 = 1000;

#[repr(u64)]
enum TglPebs {
    L3_miss = 0xd1 | (0x20 << 8),
    All_stores = 0xd0 | (0x82 << 8),
}

#[repr(u64)]
enum SkxPebs {
    L3_miss = 0x4f | (0x01 << 8),
    All_stores = 0x4f | (0x02 << 8),
}

#[derive(Parser)]
struct Args {
    #[clap(short, long, default_value = "0")]
    pid: i32,
    #[clap(short, long, default_value = "-1")]
    cpu: i32,
    #[clap(short, long, default_value = "tgl")]
    uarch: String,
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

//#[derive(Copy, Clone, Debug, PartialEq)]
//struct AccessTracker {
//    ewma: f64,
//    page: u64,
//}
//
//const ALPHA: f64 = 0.2;
//
//impl AccessTracker {
//    fn new(page: u64) -> Self {
//        Self { ewma: 0.0, page }
//    }
//
//    fn update(&mut self, new_value: f64) {
//        //self.ewma = (1.0 - ALPHA) * self.ewma + ALPHA * new_value;
//        self.ewma += new_value;
//    }
//}
//
//impl Ord for AccessTracker {
//    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//        self.ewma.partial_cmp(&other.ewma).unwrap()
//    }
//}
//
//impl PartialOrd for AccessTracker {
//    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//        Some(self.cmp(other))
//    }
//}
//
//impl Eq for AccessTracker {}

// Caller must ensure ptr is valid.
fn process_sample(sample: *const u8) {
    unsafe {
        let sample: *const demo_record = sample.cast();
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

fn build_mem_event(event: u64, disabled: bool) -> perf_event_attr {
    let mut attr = perf_event_attr::default();
    attr.type_ = perf_type_id_PERF_TYPE_RAW;
    attr.config = event;
    attr.set_sample_period(SAMPLE_PERIOD);
    attr.sample_type = perf_event_sample_format_PERF_SAMPLE_IDENTIFIER
        | perf_event_sample_format_PERF_SAMPLE_IP
        | perf_event_sample_format_PERF_SAMPLE_TID
        | perf_event_sample_format_PERF_SAMPLE_TIME
        | perf_event_sample_format_PERF_SAMPLE_ADDR;
    attr.__bindgen_anon_2.wakeup_events = (SAMPLE_PERIOD / 4) as u32;
    //attr.set_watermark(1); // Set this for wakeup watermark
    if disabled {
        attr.set_disabled(1);
    }
    attr.set_exclude_hv(1);
    attr.set_exclude_callchain_user(1);
    attr.set_exclude_callchain_kernel(1);
    attr.set_precise_ip(2);
    attr
}

fn main() {
    env_logger::init();
    let args = Args::parse();
    let l3_miss = match args.uarch.as_str() {
        "tgl" => TglPebs::L3_miss as u64,
        "skx" => SkxPebs::L3_miss as u64,
        _ => {
            error!("Unknown uarch");
            return;
        }
    };
    debug!("Raw Event: {:#02x}", l3_miss);
    let all_stores = match args.uarch.as_str() {
        "tgl" => TglPebs::All_stores as u64,
        "skx" => SkxPebs::All_stores as u64,
        _ => {
            error!("Unknown uarch");
            return;
        }
    };
    debug!("Raw Event: {:#02x}", all_stores);
    let mem_read = build_mem_event(l3_miss, true);
    let mem_store = build_mem_event(all_stores, false);
    let group_leader = PerfEvent::new(mem_read, args.pid, args.cpu, None, 0).unwrap();
    let _group_follow = PerfEvent::new(
        mem_store,
        args.pid,
        args.cpu,
        Some(group_leader.get_fd()),
        0,
    )
    .unwrap();
    group_leader.reset().unwrap();
    group_leader.enable().unwrap();
    group_leader
        .sample_loop(process_sample, size_of::<demo_record>())
        .unwrap();
}

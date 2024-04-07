#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)] // For perf bindings

use std::collections::{BinaryHeap, HashMap};

use clap::Parser;
use log::{debug, error};
use perf::perf_sys::{
    perf_event_attr, perf_event_header, perf_event_sample_format_PERF_SAMPLE_ADDR,
    perf_event_sample_format_PERF_SAMPLE_TID, perf_event_sample_format_PERF_SAMPLE_TIME,
    perf_type_id_PERF_TYPE_RAW,
};

use crate::perf::{perf_event_sample_format_PERF_SAMPLE_IP, perf_event_type_PERF_RECORD_SAMPLE};

mod perf;

const SAMPLE_FREQ: u64 = 4000;
const SAMPLE_PERIOD: u64 = 10000;
const MMAP_PAGES: usize = 1 + (1 << 16); //(must be power of 2) + 1

enum TglPebs {
    L3_miss = 0xd1 | (0x20 << 8),
    All_stores = 0xd0 | (0x82 << 8),
}

enum SkxPebs {
    L3_miss = 0x4f | (0x01 << 8),
    All_stores = 0x4f | (0x02 << 8),
}

#[derive(Parser)]
struct Args {
    #[clap(short, long, default_value = "0")]
    pid: u32,
    #[clap(short, long, default_value = "-1")]
    cpu: i32,
    #[clap(short, long, default_value = "tgl")]
    uarch: String,
}

#[derive(Default, Debug)]
#[repr(C)]
struct demo_record {
    hdr: perf_event_header,
    ip: u64,
    tid: u64,
    time: u64,
    addr: u64,
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct AccessTracker {
    ewma: f64,
    page: u64,
}

const ALPHA: f64 = 0.2;

impl AccessTracker {
    fn new(page: u64) -> Self {
        Self { ewma: 0.0, page }
    }

    fn update(&mut self, new_value: f64) {
        //self.ewma = (1.0 - ALPHA) * self.ewma + ALPHA * new_value;
        self.ewma += new_value;
    }
}

impl Ord for AccessTracker {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ewma.partial_cmp(&other.ewma).unwrap()
    }
}

impl PartialOrd for AccessTracker {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for AccessTracker {}

unsafe fn read_samples(mmap: *mut perf::perf_event_mmap_page) {
    let mut stat = HashMap::new();
    let mut heap = BinaryHeap::new();
    println!("tid,time,addr,cpu,phys_addr");
    let data_begin = mmap.byte_add((*mmap).data_offset as usize);
    debug!("Data offset: {:#02x}", (*mmap).data_offset);
    debug!("Data size: {:#02x}", (*mmap).data_size);
    debug!("Data begin: {:p}", data_begin);
    debug!("Region begin: {:p}", mmap);
    let mut flag = false;
    loop {
        let next = ((*mmap).data_tail % (*mmap).data_size) as usize;
        let hdr: *const perf_event_header = data_begin.byte_add(next).cast();
        match (*hdr).type_ {
            perf_event_type_PERF_RECORD_SAMPLE => {
                let sample: *const demo_record = hdr.cast();
                if (*sample).addr == 0 {
                    continue;
                }
                let page = (*sample).addr & !4095;
                let record = stat.entry(page).or_insert_with(|| {
                    flag = true;
                    debug!("Sample: {:#02x}", page);
                    AccessTracker::new(page)
                });
                (*record).update(1.0);
                if flag {
                    heap.push(*record);
                    flag = false;
                }
            }
            _ => {
                //debug!("Unknown record type: {}", (*hdr).type_);
            }
        }
        (*mmap).data_tail += (*hdr).size as u64;
    }
}

fn main() {
    env_logger::init();
    let args = Args::parse();
    let l3_miss = match args.uarch.as_str() {
        "tgl" => Some(TglPebs::L3_miss as u64),
        "skx" => Some(SkxPebs::L3_miss as u64),
        _ => {
            error!("Unknown uarch");
            None
        }
    };
    debug!("Raw Event: {:#02x}", l3_miss.unwrap());
    //let all_stores = match args.uarch.as_str() {
    //    "tgl" => Some(TglPebs::All_stores as u64),
    //    "skx" => Some(SkxPebs::All_stores as u64),
    //    _ => {
    //        error!("Unknown uarch");
    //        None
    //    }
    //};
    let mut attr = perf_event_attr::default();
    attr.type_ = perf_type_id_PERF_TYPE_RAW;
    attr.config = if let Some(reads) = l3_miss {
        reads
    } else {
        error!("Unknown uarch");
        panic!();
    };
    attr.sample_period(SAMPLE_PERIOD);
    attr.sample_type = perf_event_sample_format_PERF_SAMPLE_IP
        | perf_event_sample_format_PERF_SAMPLE_TID
        | perf_event_sample_format_PERF_SAMPLE_TIME
        | perf_event_sample_format_PERF_SAMPLE_ADDR;
    //attr.__bindgen_anon_2.wakeup_events = (SAMPLE_PERIOD / 2) as u32;
    //attr.set_watermark(1);
    attr.set_exclude_hv(1);
    attr.set_exclude_callchain_user(1);
    attr.set_exclude_callchain_kernel(1);
    attr.set_precise_ip(2);
    let event_fd = perf::perf_event_open(attr, args.pid as i32, args.cpu, 0).unwrap();
    let mmap = unsafe { perf::mmap_perf_buffer(event_fd, MMAP_PAGES).unwrap() };
    unsafe { read_samples(mmap) };
}

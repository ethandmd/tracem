#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)] // For perf bindings

use clap::Parser;
use log::{debug, error};
use perf::perf_sys::{
    perf_event_attr, perf_event_header, perf_event_sample_format_PERF_SAMPLE_ADDR,
    perf_event_sample_format_PERF_SAMPLE_PHYS_ADDR, perf_event_sample_format_PERF_SAMPLE_TID,
    perf_event_sample_format_PERF_SAMPLE_TIME, perf_type_id_PERF_TYPE_RAW,
};

use crate::perf::{
    perf_event_type_PERF_RECORD_EXIT, perf_event_type_PERF_RECORD_LOST,
    perf_event_type_PERF_RECORD_SAMPLE,
};

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
    tid: u64,
    time: u64,
    addr: u64,
    cpu: u32,
    phys_addr: u64,
}

unsafe fn read_samples(mmap: *mut perf::perf_event_mmap_page) {
    println!("tid,time,addr,cpu,phys_addr");
    let mut lost = 0;
    let data_begin = mmap.byte_add((*mmap).offset as usize);
    debug!("Data offset: {:#02x}", (*mmap).offset);
    debug!("Data size: {:#02x}", (*mmap).data_size);
    debug!("Data begin: {:p}", data_begin);
    loop {
        let next = ((*mmap).data_tail % (*mmap).data_size) as usize;
        let hdr: *const perf_event_header = data_begin.byte_add(next).cast();
        debug!("Header type: {}", (*hdr).type_);
        match (*hdr).type_ {
            perf_event_type_PERF_RECORD_SAMPLE => {
                // 9
                let sample: *const demo_record = hdr.cast();
                if (*sample).addr != 0 {
                    println!(
                        "{},{},{},{},{}",
                        (*sample).tid,
                        (*sample).time,
                        (*sample).addr,
                        (*sample).cpu,
                        (*sample).phys_addr
                    );
                }
            }
            perf_event_type_PERF_RECORD_EXIT => {
                debug!("Exit record");
            }
            perf_event_type_PERF_RECORD_LOST => {
                lost += 1;
                debug!("Lost samples: {}", lost);
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
    attr.sample_type = perf_event_sample_format_PERF_SAMPLE_TID
        | perf_event_sample_format_PERF_SAMPLE_TIME
        | perf_event_sample_format_PERF_SAMPLE_ADDR
        | perf_event_sample_format_PERF_SAMPLE_PHYS_ADDR;
    attr.set_exclude_hv(1);
    attr.set_exclude_callchain_user(1);
    attr.set_exclude_callchain_kernel(1);
    attr.set_precise_ip(2);
    let event_fd = perf::perf_event_open(attr, args.pid as i32, args.cpu, 0).unwrap();
    let mmap = unsafe { perf::mmap_perf_buffer(event_fd, MMAP_PAGES).unwrap() };
    unsafe { read_samples(mmap) };
}

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)] // For perf bindings

use std::env::args;

use log::{debug, info};
use perf::*;

mod perf;

const SAMPLE_FREQ: u64 = 4000;
const SAMPLE_PERIOD: u64 = 1000;
const MMAP_SIZE: usize = 1 + (1 << 16) * 4096; // 1Mib (must be power of 2) + 1

#[derive(Default)]
struct demo_record {
    hdr: perf::perf_event_header,
    tid: u64,
    time: u64,
}

fn main() {
    let pid = args().nth(1).map(|pid| pid.parse().unwrap()).unwrap_or(0);
    let cpu = args().nth(2).map(|cpu| cpu.parse().unwrap()).unwrap_or(-1);
    env_logger::init();
    //let event = PerfEventBuilder::default()
    //    .type_id(TypeId::HardwareCache)
    //    .type_config(
    //        PerfHwCacheConfigBuilder::new()
    //            .cache_id(PerfHwCacheId::LL)
    //            .op_id(PerfHwCacheOpId::Read)
    //            .result_id(PerfHwCacheOpResultId::Miss),
    //    )
    //    .unwrap()
    //    .sample_freq(SAMPLE_FREQ)
    //    .unwrap()
    //    .sample_format(&[
    //        SampleFormat::Tid,
    //        SampleFormat::Time,
    //        SampleFormat::Addr,
    //        SampleFormat::Weight,
    //    ])
    //    .unwrap()
    //    //.wakeup_n_events(100)
    //    //.unwrap()
    //    .flags(&[
    //        EventAttrFlags::ExcludeKernel,
    //        EventAttrFlags::ExcludeHV,
    //        EventAttrFlags::ExcludeCallchainUser,
    //        EventAttrFlags::ExcludeKernel,
    //        EventAttrFlags::PreciseIpConstantSkid,
    //    ])
    //    .build(pid, cpu, None)
    //    .unwrap();
    let event = PerfEventBuilder::default()
        .type_id(TypeId::Hardware)
        .type_config(PerfHwConfigBuilder::hw_cpu_cycles())
        .unwrap()
        .sample_freq(SAMPLE_FREQ)
        .unwrap()
        .sample_format(&[SampleFormat::Tid, SampleFormat::Time])
        .unwrap()
        .flags(&[EventAttrFlags::Disabled])
        .build(pid, cpu, None)
        .unwrap();
    let buf = event.mmap_buffer(MMAP_SIZE).unwrap();
    debug!("Perf mmap version: {}", buf.version());
    debug!("Perf mmap data size: 0x{:x}", buf.data_size());
    debug!("Perf mmap data head: 0x{:x}", buf.data_head());
    debug!("Perf mmap data tail: 0x{:x}", buf.data_tail());
    debug!("Perf mmap time enabled: 0x{:x}", buf.time_enabled());
    debug!("Perf mmap time running: 0x{:x}", buf.time_running());
    debug!("perf mmap event index: 0x{:x}", buf.index());
    event.reset().unwrap();
    event.enable().unwrap();
    let xs = vec![1.0; 1 << 20];
    let ys = vec![2.0; 1 << 20];
    let mut zs = vec![0.0; 1 << 20];
    for i in 0..(1 << 20) {
        zs[i] = 0.5 * xs[i] + ys[i];
    }
    event.disable().unwrap();
    debug!("Perf mmap version: {}", buf.version());
    debug!("Perf mmap data size: 0x{:x}", buf.data_size());
    debug!("Perf mmap data head: 0x{:x}", buf.data_head());
    debug!("Perf mmap data tail: 0x{:x}", buf.data_tail());
    debug!("Perf mmap time enabled: 0x{:x}", buf.time_enabled());
    debug!("Perf mmap time running: 0x{:x}", buf.time_running());
    debug!("perf mmap event index: 0x{:x}", buf.index());
    let mut record = demo_record::default();
    for _ in 0..10 {
        let _ = unsafe { buf.read_sample(&mut record) };
        info!("TID: {}, Time: {}", record.tid, record.time);
    }
}

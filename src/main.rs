#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)] // For perf bindings

use log::{debug, info};
use perf::*;

mod perf;

const SAMPLE_PERIOD: u64 = 10000;
const MMAP_SIZE: usize = 1 + (1 << 16); // 1Mib (must be power of 2) + 1

#[derive(Default)]
struct l3_miss_record {
    hdr: perf::perf_event_header,
    tid: u64,
    time: u64,
    addr: u64,
    weight: u64,
}

fn build_l3_cache_miss() -> Result<PerfEventHandle, perf::BuilderError> {
    let pid = 0;
    let cpu = -1;
    PerfEventBuilder::new()
        .type_id(TypeId::HardwareCache)
        .type_config(
            PerfHwCacheConfigBuilder::new()
                .cache_id(PerfHwCacheId::LL)
                .op_id(PerfHwCacheOpId::Read)
                .result_id(PerfHwCacheOpResultId::Miss),
        )
        .unwrap()
        .sample_period(SAMPLE_PERIOD)
        .unwrap()
        .sample_format(&[
            SampleFormat::Tid,
            SampleFormat::Time,
            SampleFormat::Addr,
            SampleFormat::Weight,
        ])
        .unwrap()
        .wakeup_n_events(100)
        .unwrap()
        .flags(&[
            EventAttrFlags::ExcludeKernel,
            EventAttrFlags::ExcludeHV,
            EventAttrFlags::ExcludeCallchainUser,
            EventAttrFlags::ExcludeKernel,
            EventAttrFlags::PreciseIpConstantSkid,
        ])
        .build(pid, cpu, None)
}

fn main() {
    env_logger::init();
    let event = build_l3_cache_miss().unwrap();
    event.enable().unwrap();
    let buf = event.mmap_buffer(MMAP_SIZE).unwrap();
    debug!("Perf mmap version: {}", buf.version());
    let mut record = l3_miss_record::default();
    for _ in 0..10 {
        let _ = unsafe { buf.read_sample(&mut record) };
        info!(
            "TID: {}, Time: {}, Addr: {}, Weight: {}",
            record.tid, record.time, record.addr, record.weight
        );
    }
}

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)] // For perf bindings

use perf::*;

mod perf;

const SAMPLE_PERIOD: u64 = 10000;

fn build_l3_cache_miss() -> Result<PerfEventHandle, perf::EventOpenError> {
    let builder = PerfEventBuilder::new()
        .type_id(TypeId::HardwareCache)
        .type_config(
            PerfHwCacheConfigBuilder::new()
                .cache_id(PerfHwCacheId::Node)
                .op_id(PerfHwCacheOpId::Read)
                .result_id(PerfHwCacheOpResultId::Access),
        )
        .unwrap()
        .sample_period(SAMPLE_PERIOD)
        .unwrap()
        .sample_format(&[
            SampleFormat::Ip,
            SampleFormat::Tid,
            SampleFormat::Time,
            SampleFormat::Addr,
            SampleFormat::Weight,
        ])
        .unwrap()
        .flags(&[
            EventAttrFlags::ExcludeKernel,
            EventAttrFlags::ExcludeHV,
            EventAttrFlags::ExcludeCallchainUser,
            EventAttrFlags::ExcludeKernel,
            EventAttrFlags::PreciseIpConstantSkid,
        ]);
    let pid = 0;
    let cpu = -1;

    perf::perf_event_open(builder, pid, cpu, None)
}

fn main() {
    let _l3_miss = build_l3_cache_miss().unwrap();
}

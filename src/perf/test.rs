use self::perf_sys::{
    perf_hw_cache_id_PERF_COUNT_HW_CACHE_LL, perf_hw_cache_op_id_PERF_COUNT_HW_CACHE_OP_READ,
    perf_hw_cache_op_result_id_PERF_COUNT_HW_CACHE_RESULT_MISS,
};

use super::*;

#[test]
fn test_event_type_id_hw_cache() {
    assert_eq!(
        TypeId::HardwareCache.to_perf_sys(),
        perf_sys::perf_type_id_PERF_TYPE_HW_CACHE
    );
    let b = PerfEventBuilder::new().type_id(TypeId::HardwareCache);
    assert_eq!(b.attr.type_, TypeId::HardwareCache.to_perf_sys());
}

#[test]
fn test_event_config_hw_cache() {
    let c = perf_hw_cache_id_PERF_COUNT_HW_CACHE_LL
        | (perf_hw_cache_op_id_PERF_COUNT_HW_CACHE_OP_READ << 8)
        | (perf_hw_cache_op_result_id_PERF_COUNT_HW_CACHE_RESULT_MISS << 16);
    let ll = PerfHwCacheId::LL;
    assert_eq!(
        ll.to_perf_sys().unwrap(),
        perf_hw_cache_id_PERF_COUNT_HW_CACHE_LL
    );
    let b = PerfEventBuilder::new()
        .type_config(
            PerfHwCacheConfigBuilder::new()
                .cache_id(PerfHwCacheId::LL)
                .op_id(PerfHwCacheOpId::Read)
                .result_id(PerfHwCacheOpResultId::Miss),
        )
        .unwrap();
    assert_eq!(b.attr.config, c as u64);
}

#[test]
fn test_event_sample_period() {
    let period = 10000;
    let b = PerfEventBuilder::new().sample_period(period).unwrap();
    assert_eq!(unsafe { b.attr.__bindgen_anon_1.sample_period }, period);
    assert_eq!(b.attr.freq(), 0);
}

#[test]
fn test_event_sample_format() {
    let format = &[
        SampleFormat::Tid,
        SampleFormat::Time,
        SampleFormat::Ip,
        SampleFormat::Addr,
    ];
    assert_eq!(
        SampleFormat::Tid.to_perf_sys().unwrap(),
        perf_sys::perf_event_sample_format_PERF_SAMPLE_TID
    );
    assert_eq!(
        SampleFormat::Time.to_perf_sys().unwrap(),
        perf_sys::perf_event_sample_format_PERF_SAMPLE_TIME
    );
    assert_eq!(
        SampleFormat::Ip.to_perf_sys().unwrap(),
        perf_sys::perf_event_sample_format_PERF_SAMPLE_IP
    );
    assert_eq!(
        SampleFormat::Addr.to_perf_sys().unwrap(),
        perf_sys::perf_event_sample_format_PERF_SAMPLE_ADDR
    );
    let fb = SampleFormat::Tid.to_perf_sys().unwrap()
        | SampleFormat::Time.to_perf_sys().unwrap()
        | SampleFormat::Ip.to_perf_sys().unwrap()
        | SampleFormat::Addr.to_perf_sys().unwrap();
    let b = PerfEventBuilder::new().sample_format(format).unwrap();
    assert_eq!(b.attr.sample_type, fb);
}

#[test]
fn test_event_flags() {
    let f = &[
        EventAttrFlags::ExcludeKernel,
        EventAttrFlags::PreciseIpConstantSkid,
    ];
    let b = PerfEventBuilder::new().flags(f);
    // ExcludeKernel is bit 5
    // PreciseIP is bits 15-16; ConstantSkid is 1
    assert_eq!(b.attr.precise_ip(), 1);
    assert_eq!(b.attr.exclude_kernel(), 1);
}

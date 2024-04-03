use bitfield::bitfield;

use self::perf_sys::{
    __BindgenBitfieldUnit, perf_event_attr, perf_event_attr__bindgen_ty_1,
    perf_event_attr__bindgen_ty_2, perf_event_attr__bindgen_ty_3, perf_event_attr__bindgen_ty_4,
};

mod perf_sys {
    include!(concat!(env!("OUT_DIR"), "/perf-sys.rs"));
}

type PerfEventConfigType = u64;
type FdType = i64;

#[derive(Debug)]
pub enum BuilderError {
    Unset,
    BitImpossible,
}

#[derive(Debug)]
pub enum EventOpenError {
    SyscallError,
}

/// Set in the type field of the perf_event_attr struct.
pub enum TypeId {
    Hardware,
    Software,
    Tracepoint,
    HardwareCache,
    Raw,
    Breakpoint,
    Max,
}

impl TypeId {
    fn to_perf_sys(&self) -> u32 {
        match self {
            TypeId::Hardware => perf_sys::perf_type_id_PERF_TYPE_HARDWARE,
            TypeId::Software => perf_sys::perf_type_id_PERF_TYPE_SOFTWARE,
            TypeId::Tracepoint => perf_sys::perf_type_id_PERF_TYPE_TRACEPOINT,
            TypeId::HardwareCache => perf_sys::perf_type_id_PERF_TYPE_HW_CACHE,
            TypeId::Raw => perf_sys::perf_type_id_PERF_TYPE_RAW,
            TypeId::Breakpoint => perf_sys::perf_type_id_PERF_TYPE_BREAKPOINT,
            TypeId::Max => perf_sys::perf_type_id_PERF_TYPE_MAX,
        }
    }
}

// From <linux/perf_event.h>
// PERF_TYPE_HW_CACHE:          0xEEEEEEEE00DDCCBB
//                   BB: hardware cache ID
//                   CC: hardware cache op ID
//                   DD: hardware cache op result ID
//                   EEEEEEEE: PMU type ID
bitfield! {
    struct PerfHwCacheConfig(u64);
    cache_id, set_cache_id: 0, 2;
    op_id, set_op_id: 2, 2;
    result_id, set_result_id: 4, 2;
}

impl From<PerfHwCacheConfig> for PerfEventConfigType {
    fn from(val: PerfHwCacheConfig) -> PerfEventConfigType {
        val.0
    }
}

pub enum PerfHwCacheId {
    L1D,
    L1I,
    LL,
    DTLB,
    ITLB,
    Bpu,
    Node,
    Max,
    _unset,
}

impl From<PerfHwCacheId> for u64 {
    fn from(val: PerfHwCacheId) -> u64 {
        match val {
            PerfHwCacheId::L1D => 0,
            PerfHwCacheId::L1I => 1,
            PerfHwCacheId::LL => 2,
            PerfHwCacheId::DTLB => 3,
            PerfHwCacheId::ITLB => 4,
            PerfHwCacheId::Bpu => 5,
            PerfHwCacheId::Node => 6,
            PerfHwCacheId::Max => 7,
            PerfHwCacheId::_unset => 8,
        }
    }
}

pub enum PerfHwCacheOpId {
    Read,
    Write,
    Prefetch,
    Max,
    _unset,
}

impl From<PerfHwCacheOpId> for u64 {
    fn from(val: PerfHwCacheOpId) -> u64 {
        match val {
            PerfHwCacheOpId::Read => 0,
            PerfHwCacheOpId::Write => 1,
            PerfHwCacheOpId::Prefetch => 2,
            PerfHwCacheOpId::Max => 3,
            PerfHwCacheOpId::_unset => 4,
        }
    }
}

pub enum PerfHwCacheOpResultId {
    Access,
    Miss,
    Max,
    _unset,
}

impl From<PerfHwCacheOpResultId> for u64 {
    fn from(val: PerfHwCacheOpResultId) -> u64 {
        match val {
            PerfHwCacheOpResultId::Access => 0,
            PerfHwCacheOpResultId::Miss => 1,
            PerfHwCacheOpResultId::Max => 2,
            PerfHwCacheOpResultId::_unset => 3,
        }
    }
}

pub struct PerfHwCacheConfigBuilder {
    hw_cache_id: PerfHwCacheId,
    op_id: PerfHwCacheOpId,
    result_id: PerfHwCacheOpResultId,
}

impl PerfHwCacheConfigBuilder {
    pub fn new() -> PerfHwCacheConfigBuilder {
        PerfHwCacheConfigBuilder {
            hw_cache_id: PerfHwCacheId::_unset,
            op_id: PerfHwCacheOpId::_unset,
            result_id: PerfHwCacheOpResultId::_unset,
        }
    }

    pub fn validate_build(&self) -> Result<(), BuilderError> {
        if let PerfHwCacheId::_unset = self.hw_cache_id {
            return Err(BuilderError::Unset);
        }
        if let PerfHwCacheOpId::_unset = self.op_id {
            return Err(BuilderError::Unset);
        }
        if let PerfHwCacheOpResultId::_unset = self.result_id {
            return Err(BuilderError::Unset);
        }
        Ok(())
    }

    pub fn cache_id(mut self, hw_cache_id: PerfHwCacheId) -> Self {
        self.hw_cache_id = hw_cache_id;
        self
    }

    pub fn op_id(mut self, op_id: PerfHwCacheOpId) -> Self {
        self.op_id = op_id;
        self
    }

    pub fn result_id(mut self, result_id: PerfHwCacheOpResultId) -> Self {
        self.result_id = result_id;
        self
    }

    fn build(self) -> Result<PerfHwCacheConfig, BuilderError> {
        if let PerfHwCacheId::_unset = self.hw_cache_id {
            return Err(BuilderError::Unset);
        }
        if let PerfHwCacheOpId::_unset = self.op_id {
            return Err(BuilderError::Unset);
        }
        if let PerfHwCacheOpResultId::_unset = self.result_id {
            return Err(BuilderError::Unset);
        }

        let mut config = PerfHwCacheConfig(0);
        config.set_cache_id(self.hw_cache_id.into());
        config.set_op_id(self.op_id.into());
        config.set_result_id(self.result_id.into());
        Ok(config)
    }
}

impl TryInto<PerfEventConfigType> for PerfHwCacheConfigBuilder {
    type Error = BuilderError;

    fn try_into(self) -> Result<PerfEventConfigType, Self::Error> {
        self.build().map(|config| config.into())
    }
}

/// Set in the sample_type field of the perf_event_attr struct.
pub enum SampleFormat {
    Ip,
    Tid,
    Time,
    Addr,
    Read,
    Callchain,
    ID,
    Cpu,
    Period,
    StreamID,
    Raw,
    BranchStack,
    RegsUser,
    StackUser,
    Weight,
    DataSrc,
    Identifier,
    Transaction,
    RegsIntr,
    PhysAddr,
    Aux,
    CGroup,
    DataPageSize,
    CodePageSize,
    WeightStruct,
    Max,
    _unset,
}

impl SampleFormat {
    fn to_perf_sys(&self) -> Result<u64, BuilderError> {
        match self {
            Self::Ip => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_IP),
            Self::Tid => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_TID),
            Self::Time => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_TIME),
            Self::Addr => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_ADDR),
            Self::Read => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_READ),
            Self::Callchain => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_CALLCHAIN),
            Self::ID => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_ID),
            Self::Cpu => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_CPU),
            Self::Period => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_PERIOD),
            Self::StreamID => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_STREAM_ID),
            Self::Raw => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_RAW),
            Self::BranchStack => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_BRANCH_STACK),
            Self::RegsUser => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_REGS_USER),
            Self::StackUser => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_STACK_USER),
            Self::Weight => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_WEIGHT),
            Self::DataSrc => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_DATA_SRC),
            Self::Identifier => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_IDENTIFIER),
            Self::Transaction => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_TRANSACTION),
            Self::RegsIntr => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_REGS_INTR),
            Self::PhysAddr => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_PHYS_ADDR),
            Self::Aux => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_AUX),
            Self::CGroup => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_CGROUP),
            Self::DataPageSize => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_DATA_PAGE_SIZE),
            Self::CodePageSize => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_CODE_PAGE_SIZE),
            Self::WeightStruct => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_WEIGHT_STRUCT),
            Self::Max => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_MAX),
            Self::_unset => Err(BuilderError::Unset),
        }
    }
}

/// Set this enum in the read_format field of the perf_event_attr struct.
pub enum ReadFormat {
    TotalTimeEnabled,
    TotalTimeRunning,
    ID,
    Group,
    Lost,
    Max,
    _unset,
}

impl ReadFormat {
    fn to_perf_sys(&self) -> Result<u64, BuilderError> {
        match self {
            Self::TotalTimeEnabled => {
                Ok(perf_sys::perf_event_read_format_PERF_FORMAT_TOTAL_TIME_ENABLED as u64)
            }
            Self::TotalTimeRunning => {
                Ok(perf_sys::perf_event_read_format_PERF_FORMAT_TOTAL_TIME_RUNNING as u64)
            }
            Self::ID => Ok(perf_sys::perf_event_read_format_PERF_FORMAT_ID as u64),
            Self::Group => Ok(perf_sys::perf_event_read_format_PERF_FORMAT_GROUP as u64),
            Self::Lost => Ok(perf_sys::perf_event_read_format_PERF_FORMAT_LOST as u64),
            Self::Max => Ok(perf_sys::perf_event_read_format_PERF_FORMAT_MAX as u64),
            Self::_unset => Err(BuilderError::Unset),
        }
    }
}

pub enum EventAttrFlags {
    Disabled,
    Inherit,
    Pinned,
    Exclusive,
    ExcludeUser,
    ExcludeKernel,
    ExcludeHV,
    ExcludeIdle,
    MMap,
    Comm,
    Freq,
    InheritStat,
    EnableOnExec,
    Task,
    Watermark,
    PreciseIpAnySkid,
    PreciseIpConstantSkid,
    PreciseIpPleaseNoSkid,
    PreciseIpNoSkid,
    MMapData,
    SampleIDAll,
    ExcludeHost,
    ExcludeGuest,
    ExcludeCallchainKernel,
    ExcludeCallchainUser,
    MMap2,
    CommExec,
    UseClockID,
    ContextSwitch,
    WriteBackward,
    Namespaces,
    KSymbol,
    BPFEvent,
    AuxOutput,
    CGroup,
    TextPoke,
    BuildID,
    InheritThread,
    RemoveOnExec,
    SigTrap,
}

impl EventAttrFlags {
    fn set_attr_bitfield(&self, attr: &mut perf_event_attr) {
        match self {
            Self::Disabled => attr.set_disabled(1),
            Self::Inherit => attr.set_inherit(1),
            Self::Pinned => attr.set_pinned(1),
            Self::Exclusive => attr.set_exclusive(1),
            Self::ExcludeUser => attr.set_exclude_user(1),
            Self::ExcludeKernel => attr.set_exclude_kernel(1),
            Self::ExcludeHV => attr.set_exclude_hv(1),
            Self::ExcludeIdle => attr.set_exclude_idle(1),
            Self::MMap => attr.set_mmap(1),
            Self::Comm => attr.set_comm(1),
            Self::Freq => attr.set_freq(1),
            Self::InheritStat => attr.set_inherit_stat(1),
            Self::EnableOnExec => attr.set_enable_on_exec(1),
            Self::Task => attr.set_task(1),
            Self::Watermark => attr.set_watermark(1),
            Self::PreciseIpAnySkid => attr.set_precise_ip(0),
            Self::PreciseIpConstantSkid => attr.set_precise_ip(1),
            Self::PreciseIpPleaseNoSkid => attr.set_precise_ip(2),
            Self::PreciseIpNoSkid => attr.set_precise_ip(3),
            Self::MMapData => attr.set_mmap_data(1),
            Self::SampleIDAll => attr.set_sample_id_all(1),
            Self::ExcludeHost => attr.set_exclude_host(1),
            Self::ExcludeGuest => attr.set_exclude_guest(1),
            Self::ExcludeCallchainKernel => attr.set_exclude_callchain_kernel(1),
            Self::ExcludeCallchainUser => attr.set_exclude_callchain_user(1),
            Self::MMap2 => attr.set_mmap2(1),
            Self::CommExec => attr.set_comm_exec(1),
            Self::UseClockID => attr.set_use_clockid(1),
            Self::ContextSwitch => attr.set_context_switch(1),
            Self::WriteBackward => attr.set_write_backward(1),
            Self::Namespaces => attr.set_namespaces(1),
            Self::KSymbol => attr.set_ksymbol(1),
            Self::BPFEvent => attr.set_bpf_event(1),
            Self::AuxOutput => attr.set_aux_output(1),
            Self::CGroup => attr.set_cgroup(1),
            Self::TextPoke => attr.set_text_poke(1),
            Self::BuildID => attr.set_build_id(1),
            Self::InheritThread => attr.set_inherit_thread(1),
            Self::RemoveOnExec => attr.set_remove_on_exec(1),
            Self::SigTrap => attr.set_sigtrap(1),
        };
    }
}

pub struct PerfEventBuilder {
    attr: perf_event_attr,
}

impl PerfEventBuilder {
    pub fn new() -> PerfEventBuilder {
        PerfEventBuilder {
            attr: perf_event_attr {
                type_: 0,
                size: std::mem::size_of::<perf_event_attr>() as u32,
                config: 0,
                __bindgen_anon_1: perf_event_attr__bindgen_ty_1 { sample_period: 0 },
                sample_type: 0,
                read_format: 0,
                _bitfield_align_1: [0; 0],
                _bitfield_1: __BindgenBitfieldUnit::new([0; 8]),
                __bindgen_anon_2: perf_event_attr__bindgen_ty_2 { wakeup_events: 0 },
                bp_type: 0,
                __bindgen_anon_3: perf_event_attr__bindgen_ty_3 { bp_addr: 0 },
                __bindgen_anon_4: perf_event_attr__bindgen_ty_4 { bp_len: 0 },
                branch_sample_type: 0,
                sample_regs_user: 0,
                sample_stack_user: 0,
                clockid: 0,
                sample_regs_intr: 0,
                aux_watermark: 0,
                sample_max_stack: 0,
                __reserved_2: 0,
                aux_sample_size: 0,
                __reserved_3: 0,
                sig_data: 0,
            },
        }
    }

    pub fn type_id(mut self, type_id: TypeId) -> Self {
        self.attr.type_ = type_id.to_perf_sys();
        self
    }

    pub fn type_config<T: TryInto<PerfEventConfigType>>(
        mut self,
        config: T,
    ) -> Result<Self, BuilderError> {
        self.attr.config = config.try_into().map_err(|_| BuilderError::Unset)?;
        Ok(self)
    }

    /// Set the sample period. If the `freq` flag is set, clear the flag
    /// and overwrite the sample frequency with the sample period.
    pub fn sample_period(mut self, sample_period: u64) -> Result<Self, BuilderError> {
        match self.attr.freq() {
            0 => {
                self.attr.__bindgen_anon_1.sample_period = sample_period;
                Ok(self)
            }
            1 => {
                self.attr.set_freq(0);
                self.attr.__bindgen_anon_1.sample_period = sample_period;
                Ok(self)
            }
            _ => Err(BuilderError::BitImpossible),
        }
    }

    /// Sets the sample frequency by first setting the `freq` flag.
    /// If the sample period has already been set, set the `freq` flag
    /// and overwrite the sample period with the sample frequency.
    pub fn sample_freq(mut self, sample_freq: u64) -> Result<Self, BuilderError> {
        match self.attr.freq() {
            0 => {
                self.attr.set_freq(1);
                self.attr.__bindgen_anon_1.sample_freq = sample_freq;
                Ok(self)
            }
            1 => {
                self.attr.__bindgen_anon_1.sample_freq = sample_freq;
                Ok(self)
            }
            _ => Err(BuilderError::BitImpossible),
        }
    }

    pub fn sample_format(mut self, sample_type: &[SampleFormat]) -> Result<Self, BuilderError> {
        for t in sample_type {
            self.attr.sample_type |= t.to_perf_sys()?;
        }
        Ok(self)
    }

    pub fn read_format(mut self, read_format: &[ReadFormat]) -> Result<Self, BuilderError> {
        for f in read_format {
            self.attr.read_format |= f.to_perf_sys()?;
        }
        Ok(self)
    }

    pub fn flags(mut self, flags: &[EventAttrFlags]) -> Self {
        for f in flags {
            f.set_attr_bitfield(&mut self.attr);
        }
        self
    }

    /// Set to wake up every n events, if the watermark flag is not set, set it
    /// and overwrite the wakeup bytes watermark.
    pub fn wakeup_n_events(mut self, n_events: u32) -> Result<Self, BuilderError> {
        // Watermark flag must be zero.
        match self.attr.watermark() {
            0 => {
                self.attr.__bindgen_anon_2.wakeup_events = n_events;
                Ok(self)
            }
            1 => {
                self.attr.set_watermark(0);
                self.attr.__bindgen_anon_2.wakeup_events = n_events;
                Ok(self)
            }
            _ => Err(BuilderError::BitImpossible),
        }
    }

    ///
    pub fn wakeup_n_bytes(mut self, n_bytes: u32) -> Result<Self, BuilderError> {
        match self.attr.watermark() {
            0 => {
                self.attr.set_watermark(1);
                self.attr.__bindgen_anon_2.wakeup_watermark = n_bytes;
                Ok(self)
            }
            1 => {
                self.attr.__bindgen_anon_2.wakeup_watermark = n_bytes;
                Ok(self)
            }
            _ => Err(BuilderError::BitImpossible),
        }
    }

    fn validate_build(&self) -> Result<(), BuilderError> {
        // TODO
        Ok(())
    }

    fn build(self) -> Result<perf_event_attr, BuilderError> {
        self.validate_build()?;
        Ok(self.attr)
    }
}

pub enum PerfEventOpenFlags {
    CloseOnExec,
    NoGroup,
    Output,
    CGroup,
}

impl PerfEventOpenFlags {
    fn to_open_flags(&self) -> u32 {
        match self {
            Self::CloseOnExec => perf_sys::PERF_FLAG_FD_CLOEXEC,
            Self::NoGroup => perf_sys::PERF_FLAG_FD_NO_GROUP,
            Self::Output => perf_sys::PERF_FLAG_FD_OUTPUT,
            Self::CGroup => perf_sys::PERF_FLAG_PID_CGROUP,
        }
    }
}

/// Wrapper around the `perf_event_open` syscall.
///
/// TODO:
/// - Support event groups
pub fn perf_event_open(
    attr: PerfEventBuilder,
    pid: i32,
    cpu: i32,
    flags: Option<&[PerfEventOpenFlags]>,
) -> Result<PerfEventHandle, EventOpenError> {
    let attr = attr.build().map_err(|_| EventOpenError::SyscallError)?;
    let flags = match flags {
        Some(flags) => flags.iter().fold(0, |acc, f| acc | f.to_open_flags()),
        None => 0,
    };
    // SAFETY: SYS_perf_event_open syscall arguments are correct and return value is checked.
    let fd = unsafe { libc::syscall(libc::SYS_perf_event_open, &attr, pid, cpu, -1, flags) };
    if fd < 0 {
        return Err(EventOpenError::SyscallError);
    }

    Ok(PerfEventHandle(fd))
}

pub struct PerfEventHandle(FdType);

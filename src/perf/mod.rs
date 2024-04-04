use std::{mem::size_of, ptr::copy_nonoverlapping};

use log::{debug, error, info};

use self::perf_sys::{
    __BindgenBitfieldUnit, perf_event_attr, perf_event_attr__bindgen_ty_1,
    perf_event_attr__bindgen_ty_2, perf_event_attr__bindgen_ty_3, perf_event_attr__bindgen_ty_4,
    perf_event_mmap_page,
};

pub use perf_sys::perf_event_header;

impl Default for perf_event_header {
    fn default() -> Self {
        perf_event_header {
            size: 0,
            misc: 0,
            type_: 0,
        }
    }
}

#[cfg(test)]
mod test;

mod perf_sys {
    include!(concat!(env!("OUT_DIR"), "/perf-sys.rs"));
}

#[cfg(feature = "libpfm")]
mod libpfm_sys {
    include!(concat!(env!("OUT_DIR"), "/libpfm-sys.rs"));
}

#[cfg(feature = "libpfm")]
mod libpfm;

type PerfEventConfigType = u64;
type FdType = i32; // libc::c_long;

#[derive(Debug)]
pub enum BuilderError {
    Unset,
    BitImpossible,
    BreakpointWrongConfig,
    BuildError,
    LibPfmInit,
    LibPfmProbe,
}

#[derive(Debug)]
pub enum LibPfmError {
    Invalid,
    NoPmu,
    NotSupported,
    PfmGetPMUError,
    Unknown,
}

#[derive(Debug)]
pub enum EventOpenError {
    SyscallError,
    MmapFailed,
    MmapInvalidSize,
    SampleBufSize,
}

/// Set in the type field of the perf_event_attr struct.
pub enum TypeId {
    Hardware,
    Software,
    Tracepoint,
    HardwareCache,
    Raw,
    Breakpoint,
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
        }
    }
}

// If type is PERF_TYPE_HW_CACHE, then we are measuring a
// hardware CPU cache event.  To calculate the appropriate
// config value, use the following equation:

// config = (perf_hw_cache_id) |
//          (perf_hw_cache_op_id << 8) |
//          (perf_hw_cache_op_result_id << 16);
//
// TODO: Impl From/Into trait instead of to_perf_sys().
pub enum PerfHwCacheId {
    L1D,
    L1I,
    LL,
    Dtlb,
    Itlb,
    Bpu,
    Node,
    _unset,
}

impl PerfHwCacheId {
    fn to_perf_sys(&self) -> Result<u32, BuilderError> {
        match self {
            PerfHwCacheId::L1D => Ok(perf_sys::perf_hw_cache_id_PERF_COUNT_HW_CACHE_L1D),
            PerfHwCacheId::L1I => Ok(perf_sys::perf_hw_cache_id_PERF_COUNT_HW_CACHE_L1I),
            PerfHwCacheId::LL => Ok(perf_sys::perf_hw_cache_id_PERF_COUNT_HW_CACHE_LL),
            PerfHwCacheId::Dtlb => Ok(perf_sys::perf_hw_cache_id_PERF_COUNT_HW_CACHE_DTLB),
            PerfHwCacheId::Itlb => Ok(perf_sys::perf_hw_cache_id_PERF_COUNT_HW_CACHE_ITLB),
            PerfHwCacheId::Bpu => Ok(perf_sys::perf_hw_cache_id_PERF_COUNT_HW_CACHE_BPU),
            PerfHwCacheId::Node => Ok(perf_sys::perf_hw_cache_id_PERF_COUNT_HW_CACHE_NODE),
            PerfHwCacheId::_unset => Err(BuilderError::Unset),
        }
    }
}

pub enum PerfHwCacheOpId {
    Read,
    Write,
    Prefetch,
    _unset,
}

impl PerfHwCacheOpId {
    fn to_perf_sys(&self) -> Result<u32, BuilderError> {
        match self {
            PerfHwCacheOpId::Read => Ok(perf_sys::perf_hw_cache_op_id_PERF_COUNT_HW_CACHE_OP_READ),
            PerfHwCacheOpId::Write => {
                Ok(perf_sys::perf_hw_cache_op_id_PERF_COUNT_HW_CACHE_OP_WRITE)
            }
            PerfHwCacheOpId::Prefetch => {
                Ok(perf_sys::perf_hw_cache_op_id_PERF_COUNT_HW_CACHE_OP_PREFETCH)
            }
            PerfHwCacheOpId::_unset => Err(BuilderError::Unset),
        }
    }
}

pub enum PerfHwCacheOpResultId {
    Access,
    Miss,
    _unset,
}

impl PerfHwCacheOpResultId {
    fn to_perf_sys(&self) -> Result<u32, BuilderError> {
        match self {
            PerfHwCacheOpResultId::Access => {
                Ok(perf_sys::perf_hw_cache_op_result_id_PERF_COUNT_HW_CACHE_RESULT_ACCESS)
            }
            PerfHwCacheOpResultId::Miss => {
                Ok(perf_sys::perf_hw_cache_op_result_id_PERF_COUNT_HW_CACHE_RESULT_MISS)
            }
            PerfHwCacheOpResultId::_unset => Err(BuilderError::Unset),
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

    fn build(self) -> Result<PerfEventConfigType, BuilderError> {
        let config = self.hw_cache_id.to_perf_sys()?
            | self.op_id.to_perf_sys()? << 8
            | self.result_id.to_perf_sys()? << 16;
        Ok(config as PerfEventConfigType)
    }
}

impl TryInto<PerfEventConfigType> for PerfHwCacheConfigBuilder {
    type Error = BuilderError;

    fn try_into(self) -> Result<PerfEventConfigType, Self::Error> {
        self.build()
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
            Self::_unset => Err(BuilderError::Unset),
        }
    }
}

pub enum ReadFormat {
    TotalTimeEnabled,
    TotalTimeRunning,
    ID,
    Group,
    Lost,
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

impl Default for PerfEventBuilder {
    fn default() -> PerfEventBuilder {
        #[cfg(feature = "libpfm")]
        {
            info!("Initializing libpfm...");
            match unsafe { libpfm_sys::pfm_initialize() } {
                0 => {
                    info!("libpfm initialized successfully.");
                    match libpfm::debug_read_pmus_info() {
                        Ok(info) => {
                            debug!("Found PMUs: {}", info);
                        }
                        Err(e) => {
                            error!("Failed to read PMU info: {:?}", e);
                            error!("Continuing without libpfm support.");
                        }
                    }
                }
                _ => {
                    error!("Failed to initialize libpfm.");
                    error!("Continuing without libpfm support.");
                }
            }
        }
        let mut attr = perf_event_attr {
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
        };

        attr.set_disabled(1);

        PerfEventBuilder { attr }
    }
}
impl PerfEventBuilder {
    pub fn new() -> PerfEventBuilder {
        #[cfg(feature = "libpfm")]
        {
            info!("Initializing libpfm...");
            match unsafe { libpfm_sys::pfm_initialize() } {
                0 => {
                    info!("libpfm initialized successfully.");
                    match libpfm::debug_read_pmus_info() {
                        Ok(info) => {
                            debug!("Found PMUs: {}", info);
                        }
                        Err(e) => {
                            error!("Failed to read PMU info: {:?}", e);
                            error!("Continuing without libpfm support.");
                        }
                    }
                }
                _ => {
                    error!("Failed to initialize libpfm.");
                    error!("Continuing without libpfm support.");
                }
            }
        }
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
        self.attr.config = config.try_into().map_err(|_| {
            error!("Failed to convert config to PerfEventConfigType.");
            BuilderError::Unset
        })?;
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
        // TODO: Add more validation checks.
        if self.attr.type_ == TypeId::Breakpoint.to_perf_sys() && self.attr.config != 0 {
            return Err(BuilderError::BreakpointWrongConfig);
        }
        Ok(())
    }

    pub fn build(
        self,
        pid: i32,
        cpu: i32,
        flags: Option<&[PerfEventOpenFlags]>,
    ) -> Result<PerfEventHandle, BuilderError> {
        self.validate_build()?;
        perf_event_open(self.attr, pid, cpu, flags).map_err(|_| BuilderError::BuildError)
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

// Wrapper around the `perf_event_open` syscall.
//
// TODO:
// - Support event groups
fn perf_event_open(
    attr: perf_event_attr,
    pid: i32,
    cpu: i32,
    flags: Option<&[PerfEventOpenFlags]>,
) -> Result<PerfEventHandle, EventOpenError> {
    let group = -1;
    let flags = match flags {
        Some(flags) => flags.iter().fold(0, |acc, f| acc | f.to_open_flags()),
        None => 0,
    };
    debug!(
        "Opening perf event on pid: {}, cpu: {}, with flags: {}",
        pid, cpu, flags
    );
    // SAFETY: SYS_perf_event_open syscall arguments are correct and return value is checked.
    let fd =
        unsafe { libc::syscall(libc::SYS_perf_event_open, &attr, pid, cpu, group, flags) as i32 };
    if fd < 0 {
        debug!("Consider setting the `perf_event_paranoid` sysctl or granting CAP_SYS_PTRACE capabality.");
        return Err(EventOpenError::SyscallError);
    }

    Ok(PerfEventHandle(fd))
}

pub struct PerfEventHandle(FdType);

impl PerfEventHandle {
    pub fn enable(&self) -> Result<(), EventOpenError> {
        // SAFETY: Struct owns the fd and it is safe to enable it with valid ioctl.
        match unsafe { libc::ioctl(self.0, perf_sys::perf_ioc_ENABLE as u64) } {
            0 => {
                debug!("Enabled perf event.");
                Ok(())
            }
            _ => {
                error!("Failed to enable perf event.");
                Err(EventOpenError::SyscallError)
            }
        }
    }
    pub fn mmap_buffer(&self, mmap_size: usize) -> Result<PerfMmapBuf, EventOpenError> {
        // mmap size must be 1 + 2^n pages. First page is header page.
        if !(mmap_size - 1).is_power_of_two() {
            return Err(EventOpenError::MmapInvalidSize);
        }
        // SAFETY: Valid perf event fd owned by struct.
        let mmap = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                mmap_size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                self.0,
                0,
            )
        };
        if mmap == libc::MAP_FAILED {
            return Err(EventOpenError::MmapFailed);
        }
        info!("Mapped perf event buffer at {:p}", mmap);
        // SAFETY: mmap ptr is valid at this point. And ownership is transferred to the mmap buffer.
        let mmpage: *mut perf_event_mmap_page = mmap.cast();
        Ok(PerfMmapBuf(mmpage))
    }
}

impl Drop for PerfEventHandle {
    fn drop(&mut self) {
        // SAFETY: Struct owns the fd and it is safe to close it with valid ioctl.
        unsafe {
            libc::ioctl(self.0, perf_sys::perf_ioc_DISABLE as u64);
        }
    }
}

// data_head continuously increases; manually wrap by size of mmap_buffer.
// data_tail should be written to, to reflect last read data (if PROT_WRITE).
// data_offset is where in the mmap buf the perf sample data begins.
// data_size is the size of the perf sample data region within mmap buf.
//
// Data offset should start at 1st page, after hdr page. The remaining 2^n
// are the ring buffer.
pub struct PerfMmapBuf(*mut perf_event_mmap_page);

impl PerfMmapBuf {
    pub fn version(&self) -> u32 {
        // SAFETY: Mmap buffer is valid and contains the version field.
        unsafe { (*self.0).version }
    }

    pub fn data_size(&self) -> u64 {
        // SAFETY: Mmap buffer is valid and contains the data_size field.
        unsafe { (*self.0).data_size }
    }

    fn data_ptr(&self) -> *mut u8 {
        // SAFETY: Mmap buffer is valid and contains the data_offset field.
        unsafe { self.0.byte_add((*self.0).data_offset as usize).cast() }
    }

    fn wrapped_data_tail(&self) -> u64 {
        // SAFETY: Mmap buffer is valid and modding out by data size
        // (power of 2) keeps us within the mmap buffer.
        unsafe { (*self.0).data_tail & ((*self.0).data_size - 1) }
    }

    fn next_sample(&self) -> *mut perf_event_header {
        // SAFETY: Mmap buffer is valid and contains the data_offset field.
        unsafe {
            self.data_ptr()
                .add(self.wrapped_data_tail() as usize)
                .cast()
        }
    }

    /// SAFETY: Caller must ensure that T has size for perf header + sample.
    /// 1. src is valid for reads size_of::<T>() bytes based on size checks.
    /// 2. dst is valid for writes size_of::<T>() bytes based on size checks.
    /// 3. src is aligned by perf in mmap buffer, dst is aligned by caller.
    /// 4. Caller must ensure the memory regions do not overlap.
    // Should T be Copy as well?
    pub unsafe fn read_sample<T: Sized>(&self, sample: &mut T) -> Result<(), EventOpenError> {
        let next_sample = self.next_sample();
        let size = size_of::<T>() as u16;
        // SAFETY: next_sample is a valid pointer within the mmap buffer.
        let next_size = (*next_sample).size;
        if next_size < size {
            debug!(
                "Sample recv is bigger ({}B) than next perf sample ({}B).",
                size, next_size
            );
            return Err(EventOpenError::SampleBufSize);
        }

        // Equivalent of memcpy
        copy_nonoverlapping(next_sample.cast(), sample as *mut T, size as usize);
        // Update data tail to reflect read sample.
        (*self.0).data_tail += next_size as u64;
        Ok(())
    }
}

use bitfield::bitfield;

mod perf_sys {
    include!(concat!(env!("OUT_DIR"), "/perf-sys.rs"));
}
#[derive(Debug)]
pub enum BuilderError {
    TypeIdUnset,
    SamplePeriodAndFreqSet,
    WakeupEventsandWatermarkSet,
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
    _unset,
}

impl TypeId {
    fn to_perf_sys(&self) -> Result<u32, BuilderError> {
        match self {
            TypeId::Hardware => Ok(perf_sys::perf_type_id_PERF_TYPE_HARDWARE),
            TypeId::Software => Ok(perf_sys::perf_type_id_PERF_TYPE_SOFTWARE),
            TypeId::Tracepoint => Ok(perf_sys::perf_type_id_PERF_TYPE_TRACEPOINT),
            TypeId::HardwareCache => Ok(perf_sys::perf_type_id_PERF_TYPE_HW_CACHE),
            TypeId::Raw => Ok(perf_sys::perf_type_id_PERF_TYPE_RAW),
            TypeId::Breakpoint => Ok(perf_sys::perf_type_id_PERF_TYPE_BREAKPOINT),
            TypeId::Max => Ok(perf_sys::perf_type_id_PERF_TYPE_MAX),
            TypeId::_unset => Err(BuilderError::TypeIdUnset),
        }
    }
}

/// Set in the sample_type field of the perf_event_attr struct.
pub enum SampleType {
    IP,
    TID,
    Time,
    Addr,
    Read,
    Callchain,
    ID,
    CPU,
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

impl SampleType {
    fn to_perf_sys(&self) -> Result<u64, BuilderError> {
        match self {
            Self::IP => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_IP),
            Self::TID => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_TID),
            Self::Time => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_TIME),
            Self::Addr => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_ADDR),
            Self::Read => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_READ),
            Self::Callchain => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_CALLCHAIN),
            Self::ID => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_ID),
            Self::CPU => Ok(perf_sys::perf_event_sample_format_PERF_SAMPLE_CPU),
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
            Self::_unset => Err(BuilderError::TypeIdUnset),
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
            Self::_unset => Err(BuilderError::TypeIdUnset),
        }
    }
}

/// Flags for the event_attr struct.
/// PreciseIPSkid = 0 => arbitrary skid
/// PreciseIPNoSkid = 1 => constant skid
/// PreciseIPNoSkid = 2 => request no skid
/// PreciseIPNoSkid = 3 => require no skid
#[repr(u64)]
pub enum EventAttrFlags {
    Disabled = 0,
    Inherit = 1,
    Pinned = 2,
    Exclusive = 3,
    ExcludeUser = 4,
    ExcludeKernel = 5,
    ExcludeHV = 6,
    ExcludeIdle = 7,
    MMap = 8,
    Comm = 9,
    Freq = 10,
    InheritStat = 11,
    EnableOnExec = 12,
    Task = 13,
    Watermark = 14,
    PreciseIPSkid = 15,
    PreciseIPNoSkid = 16,
    MMapData = 17,
    SampleIDAll = 18,
    ExcludeHost = 19,
    ExcludeGuest = 20,
    ExcludeCallchainKernel = 21,
    ExcludeCallchainUser = 22,
    MMap2 = 23,
    CommExec = 24,
    UseClockID = 25,
    ContextSwitch = 26,
    WriteBackward = 27,
    Namespaces = 28,
    KSymbol = 29,
    BPFEvent = 30,
    AuxOutput = 31,
    CGroup = 32,
    TextPoke = 33,
    BuildID = 34,
    InheritThread = 35,
    RemoveOnExec = 36,
    SigTrap = 37,
}

impl EventAttrFlags {
    fn set_bitfield(&self, b: &mut EventAttrFlagBits) -> Result<(), BuilderError> {
        match self {
            Self::Disabled => b.set_disabled(true),
            Self::Inherit => b.set_inherit(true),
            Self::Pinned => b.set_pinned(true),
            Self::Exclusive => b.set_exclusive(true),
            Self::ExcludeUser => b.set_exclude_user(true),
            Self::ExcludeKernel => b.set_exclude_kernel(true),
            Self::ExcludeHV => b.set_exclude_hv(true),
            Self::ExcludeIdle => b.set_exclude_idle(true),
            Self::MMap => b.set_mmap(true),
            Self::Comm => b.set_comm(true),
            Self::Freq => b.set_freq(true),
            Self::InheritStat => b.set_inherit_stat(true),
            Self::EnableOnExec => b.set_enable_on_exec(true),
            Self::Task => b.set_task(true),
            Self::Watermark => b.set_watermark(true),
            Self::PreciseIPSkid => b.set_precise_ip_skid(true),
            Self::PreciseIPNoSkid => b.set_precise_ip(true),
            Self::MMapData => b.set_mmap_data(true),
            Self::SampleIDAll => b.set_sample_id_all(true),
            Self::ExcludeHost => b.set_exclude_host(true),
            Self::ExcludeGuest => b.set_exclude_guest(true),
            Self::ExcludeCallchainKernel => b.set_exclude_callchain_kernel(true),
            Self::ExcludeCallchainUser => b.set_exclude_callchain_user(true),
            Self::MMap2 => b.set_mmap2(true),
            Self::CommExec => b.set_comm_exec(true),
            Self::UseClockID => b.set_use_clockid(true),
            Self::ContextSwitch => b.set_context_switch(true),
            Self::WriteBackward => b.set_write_backward(true),
            Self::Namespaces => b.set_namespaces(true),
            Self::KSymbol => b.set_ksymbol(true),
            Self::BPFEvent => b.set_bpf_event(true),
            Self::AuxOutput => b.set_aux_output(true),
            Self::CGroup => b.set_cgroup(true),
            Self::TextPoke => b.set_text_poke(true),
            Self::BuildID => b.set_build_id(true),
            Self::InheritThread => b.set_inherit_thread(true),
            Self::RemoveOnExec => b.set_remove_on_exec(true),
            Self::SigTrap => b.set_sigtrap(true),
        };
        Ok(())
    }
}

bitfield! {
    pub struct EventAttrFlagBits(u64);
    disabled, set_disabled: 0;
    inherit, set_inherit: 1;
    pinned, set_pinned: 2;
    exclusive, set_exclusive: 3;
    exclude_user, set_exclude_user: 4;
    exclude_kernel, set_exclude_kernel: 5;
    exclude_hv, set_exclude_hv: 6;
    exclude_idle, set_exclude_idle: 7;
    mmap, set_mmap: 8;
    comm, set_comm: 9;
    freq, set_freq: 10;
    inherit_stat, set_inherit_stat: 11;
    enable_on_exec, set_enable_on_exec: 12;
    task, set_task: 13;
    watermark, set_watermark: 14;
    precise_ip, set_precise_ip: 15;
    precise_ip_skid, set_precise_ip_skid: 16;
    mmap_data, set_mmap_data: 17;
    sample_id_all, set_sample_id_all: 18;
    exclude_host, set_exclude_host: 19;
    exclude_guest, set_exclude_guest: 20;
    exclude_callchain_kernel, set_exclude_callchain_kernel: 21;
    exclude_callchain_user, set_exclude_callchain_user: 22;
    mmap2, set_mmap2: 23;
    comm_exec, set_comm_exec: 24;
    use_clockid, set_use_clockid: 25;
    context_switch, set_context_switch: 26;
    write_backward, set_write_backward: 27;
    namespaces, set_namespaces: 28;
    ksymbol, set_ksymbol: 29;
    bpf_event, set_bpf_event: 30;
    aux_output, set_aux_output: 31;
    cgroup, set_cgroup: 32;
    text_poke, set_text_poke: 33;
    build_id, set_build_id: 34;
    inherit_thread, set_inherit_thread: 35;
    remove_on_exec, set_remove_on_exec: 36;
    sigtrap, set_sigtrap: 37;
}

#[repr(C)]
union SampleTimeConfig {
    sample_period: u64,
    sample_freq: u64,
}

#[repr(C)]
union Wakepoint {
    events: u32,    // Wakup every n events
    watermark: u32, // Wakeup every n bytes
}

#[repr(C)]
union Breakpoint {
    bp_addr: u64,
    kprobe_func: u64,
    uprobe_path: u64,
    config1: u64,
}

#[repr(C)]
union BreakpointConfig {
    bp_len: u64,
    kprobe_addr: u64,
    uprobe_offset: u64,
    config2: u64,
}

// Ex:
//   //perf_page[i][READ] = perf_setup(0x1cd, 0x4, i);  // MEM_TRANS_RETIRED.LOAD_LATENCY_GT_4
//    //perf_page[i][READ] = perf_setup(0x81d0, 0, i);   // MEM_INST_RETIRED.ALL_LOADS
//    perf_page[i][DRAMREAD] = perf_setup(0x1d3, 0, i, DRAMREAD);      // MEM_LOAD_L3_MISS_RETIRED.LOCAL_DRAM
//    perf_page[i][NVMREAD] = perf_setup(0x80d1, 0, i, NVMREAD);     // MEM_LOAD_RETIRED.LOCAL_PMM
//    //perf_page[i][WRITE] = perf_setup(0x82d0, 0, i, WRITE);    // MEM_INST_RETIRED.ALL_STORES
//    //perf_page[i][WRITE] = perf_setup(0x12d0, 0, i);   // MEM_INST_RETIRED.STLB_MISS_STORES
pub struct PerfEventAttrBuilder {
    type_id: TypeId,
    config: u64,
    sample_time: SampleTimeConfig,
    sample_type: u64,
    read_format: u64,
    flags: EventAttrFlagBits,
    wakeup: Wakepoint,
    bp_type: u32,
    bp: Breakpoint,
    bp_config: BreakpointConfig,
    branch_sample_type: u64,
    sample_regs_user: u64,  // user regs to dump on samples
    sample_stack_user: u32, // size of stack to dump on samples
    clockid: u32,           // clock to use for time fields
    sample_regs_intr: u64,  // regs to dump on samples
    aux_watermark: u32,     // aux bytes before wakeup
    sample_max_stack: u32,  // max frames in callchain
    aux_sample_size: u32,
    sig_data: u64, // user data for sigtrap
}

impl PerfEventAttrBuilder {
    pub fn new() -> PerfEventAttrBuilder {
        PerfEventAttrBuilder {
            type_id: TypeId::_unset,
            config: 0,
            sample_time: SampleTimeConfig { sample_period: 0 },
            sample_type: 0,
            read_format: 0,
            flags: EventAttrFlagBits(0),
            wakeup: Wakepoint { events: 0 },
            bp_type: 0,
            bp: Breakpoint { bp_addr: 0 },
            bp_config: BreakpointConfig { bp_len: 0 },
            branch_sample_type: 0,
            sample_regs_user: 0,
            sample_stack_user: 0,
            clockid: 0,
            sample_regs_intr: 0,
            aux_watermark: 0,
            sample_max_stack: 0,
            aux_sample_size: 0,
            sig_data: 0,
        }
    }

    pub fn type_id(&mut self, type_id: TypeId) -> &mut Self {
        self.type_id = type_id;
        self
    }

    pub fn config(&mut self, config: u64) -> &mut Self {
        self.config = config;
        self
    }

    pub fn sample_period(&mut self, sample_period: u64) -> Result<&mut Self, BuilderError> {
        // SAFETY: Union is already initialized and of same type.
        match unsafe { self.sample_time.sample_period } {
            0 => {
                self.sample_time.sample_period = sample_period;
                Ok(self)
            }
            _ => Err(BuilderError::SamplePeriodAndFreqSet),
        }
    }

    pub fn sample_freq(&mut self, sample_freq: u64) -> Result<&mut Self, BuilderError> {
        // SAFETY: Union is already initialized and of same type.
        match unsafe { self.sample_time.sample_freq } {
            0 => {
                self.sample_time.sample_freq = sample_freq;
                Ok(self)
            }
            _ => Err(BuilderError::SamplePeriodAndFreqSet),
        }
    }

    pub fn sample_type(&mut self, sample_type: &[SampleType]) -> Result<&mut Self, BuilderError> {
        for t in sample_type {
            self.sample_type |= t.to_perf_sys()?;
        }
        Ok(self)
    }

    pub fn read_format(&mut self, read_format: &[ReadFormat]) -> Result<&mut Self, BuilderError> {
        for f in read_format {
            self.read_format |= f.to_perf_sys()?;
        }
        Ok(self)
    }

    pub fn set_flags(&mut self, flags: &[EventAttrFlags]) -> Result<&mut Self, BuilderError> {
        for f in flags {
            f.set_bitfield(&mut self.flags)?;
        }
        Ok(self)
    }

    pub fn wakeup_n_events(&mut self, n_events: u32) -> Result<&mut Self, BuilderError> {
        // SAFETY: Union is already initialized and of same type.
        match unsafe { self.wakeup.watermark } {
            0 => {
                self.wakeup.events = n_events;
                Ok(self)
            }
            _ => Err(BuilderError::WakeupEventsandWatermarkSet),
        }
    }

    pub fn wakeup_n_bytes(&mut self, n_bytes: u32) -> Result<&mut Self, BuilderError> {
        // SAFETY: Union is already initialized and of same type.
        match unsafe { self.wakeup.events } {
            0 => {
                self.wakeup.watermark = n_bytes;
                Ok(self)
            }
            _ => Err(BuilderError::WakeupEventsandWatermarkSet),
        }
    }

    pub fn breakpoint_type(&mut self, bp_type: u32) -> &mut Self {
        self.bp_type = bp_type;
        self
    }

    pub fn breakpoint_addr(&mut self, bp_addr: u64) -> Result<&mut Self, BuilderError> {
        // SAFETY: Union is already initialized and of same type.
        match unsafe { self.bp.kprobe_func || self.bp.uprobe_path || self.bp.config1 } {
            0 => {
                self.bp.bp_addr = bp_addr;
                Ok(self)
            }
            _ => Err(BuilderError::TypeIdUnset),
        }
    }
}

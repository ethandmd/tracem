use log::debug;

use crate::perf::libpfm_sys::{pfm_get_event_info, pfm_get_event_next, PFM_SUCCESS};

use super::{
    libpfm_sys::{
        __BindgenBitfieldUnit, pfm_event_info_t, pfm_event_info_t__bindgen_ty_1, pfm_get_pmu_info,
        pfm_os_t_PFM_OS_PERF_EVENT_EXT, pfm_pmu_info_t, pfm_pmu_info_t__bindgen_ty_1,
        pfm_pmu_type_t_PFM_PMU_TYPE_CORE, pfm_pmu_type_t_PFM_PMU_TYPE_UNCORE,
        pfm_pmu_type_t_PFM_PMU_TYPE_UNKNOWN, PFM_ERR_INVAL, PFM_ERR_NOTSUPP,
    },
    LibPfmError,
};

const NR_SUPPORTED_PMU: u32 = 384;

impl pfm_pmu_info_t {
    fn is_present(&self) -> bool {
        self.__bindgen_anon_1._bitfield_1.get_bit(0)
    }

    fn is_arch_default(&self) -> bool {
        self.__bindgen_anon_1._bitfield_1.get_bit(1)
    }

    fn is_core(&self) -> bool {
        self.__bindgen_anon_1._bitfield_1.get_bit(2)
    }

    fn is_uncore(&self) -> bool {
        self.__bindgen_anon_1._bitfield_1.get_bit(3)
    }
}

impl Default for pfm_pmu_info_t {
    fn default() -> Self {
        Self {
            name: std::ptr::null(),
            desc: std::ptr::null(),
            pmu: 0,
            type_: 0,
            size: std::mem::size_of::<pfm_pmu_info_t>(),
            nevents: 0,
            first_event: 0,
            max_encoding: 0,
            num_cntrs: 0,
            num_fixed_cntrs: 0,
            __bindgen_anon_1: pfm_pmu_info_t__bindgen_ty_1 {
                _bitfield_align_1: [0; 0],
                _bitfield_1: __BindgenBitfieldUnit::new([0; 4]),
            },
        }
    }
}

impl pfm_event_info_t {
    fn is_precise(&self) -> bool {
        self.__bindgen_anon_1._bitfield_1.get_bit(0)
    }
}

impl Default for pfm_event_info_t {
    fn default() -> Self {
        Self {
            name: std::ptr::null(),
            desc: std::ptr::null(),
            equiv: std::ptr::null(),
            size: std::mem::size_of::<pfm_event_info_t>(),
            code: 0,
            pmu: 0,
            dtype: 0,
            idx: 0,
            nattrs: 0,
            reserved: 0,
            __bindgen_anon_1: pfm_event_info_t__bindgen_ty_1 {
                _bitfield_align_1: [0; 0],
                _bitfield_1: __BindgenBitfieldUnit::new([0; 4]),
            },
        }
    }
}

fn get_pmu_info(pmu: u32) -> Result<pfm_pmu_info_t, LibPfmError> {
    let mut info = pfm_pmu_info_t::default();
    unsafe {
        match pfm_get_pmu_info(pmu, &mut info) {
            0 => Ok(info),
            PFM_ERR_NOTSUPP => Err(LibPfmError::NotSupported),
            PFM_ERR_INVAL => Err(LibPfmError::Invalid),
            _ => Err(LibPfmError::Unknown),
        }
    }
}

/// Reads the information for each PMU until get_pmu_info fails.
pub fn read_pmus_info() -> Result<Vec<pfm_pmu_info_t>, LibPfmError> {
    let mut info_vec = vec![];
    for i in 0..NR_SUPPORTED_PMU {
        if let Ok(info) = get_pmu_info(i) {
            if info.is_present() {
                info_vec.push(info);
            }
        }
    }
    if info_vec.is_empty() {
        Err(LibPfmError::NoPmu)
    } else {
        Ok(info_vec)
    }
}

/// Returns a formatted string with all PMU info in a formatted string.
#[allow(unused_variables)]
pub fn debug_read_pmus_info() -> Result<String, LibPfmError> {
    let mut fmt_str = String::new();
    let info_vec = read_pmus_info()?;
    for info in info_vec {
        let name = unsafe { std::ffi::CStr::from_ptr(info.name).to_str().unwrap() };
        let desc = unsafe { std::ffi::CStr::from_ptr(info.desc).to_str().unwrap() };
        let type_ = match info.type_ {
            pfm_pmu_type_t_PFM_PMU_TYPE_UNKNOWN => "Unknown",
            pfm_pmu_type_t_PFM_PMU_TYPE_CORE => "Core",
            pfm_pmu_type_t_PFM_PMU_TYPE_UNCORE => "Uncore",
            _ => "Unknown",
        };
        fmt_str.push_str(&format!(
            r#"
                PMU: {:?}, Name: {}
                Desc: {}
                Type: {}
                Number of events: {}
                Number of counters: {}
                Number of fixed counters: {}
                "#,
            info.pmu, name, desc, type_, info.nevents, info.num_cntrs, info.num_fixed_cntrs,
        ));
        if info.is_present() {
            let mut idx = info.first_event;
            // Loop Termination: pfm_get_event_next returns -1 when there are no more events.
            while idx != -1 {
                let mut event_info = pfm_event_info_t::default();
                if unsafe {
                    pfm_get_event_info(idx, pfm_os_t_PFM_OS_PERF_EVENT_EXT, &mut event_info)
                } == PFM_SUCCESS as i32
                {
                    debug!(
                        "PMU[{}] {} Event: {}, Desc: {}, Precise: {}",
                        name,
                        event_info.code,
                        unsafe { std::ffi::CStr::from_ptr(event_info.name).to_str().unwrap() },
                        unsafe { std::ffi::CStr::from_ptr(event_info.desc).to_str().unwrap() },
                        event_info.is_precise()
                    );
                }
                idx = unsafe { pfm_get_event_next(idx) };
            }
        }
    }
    Ok(fmt_str)
}

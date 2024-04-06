use std::mem::zeroed;

use log::{debug, error};

pub mod perf_sys {
    include!(concat!(env!("OUT_DIR"), "/perf-sys.rs"));
}

pub use perf_sys::*;

#[derive(Debug)]
pub enum PerfError {
    EventOpenError,
    MmapError,
}

impl Default for perf_event_header {
    fn default() -> Self {
        unsafe { zeroed::<Self>() }
    }
}

impl Default for perf_event_attr {
    fn default() -> Self {
        let mut hdr = unsafe { zeroed::<Self>() };
        hdr.size = std::mem::size_of::<perf_event_attr>() as u32;
        hdr
    }
}

impl perf_event_attr {
    pub fn sample_period(&mut self, period: u64) {
        match self.freq() {
            1 => self.set_freq(0),
            _ => (),
        };
        self.__bindgen_anon_1.sample_period = period;
    }
}

// TODO:
// - Support event groups
pub fn perf_event_open(
    attr: perf_event_attr,
    pid: i32,
    cpu: i32,
    flags: i32,
) -> Result<i32, PerfError> {
    let group = -1;
    debug!(
        "Opening perf event on pid: {}, cpu: {}, with flags: {}",
        pid, cpu, flags
    );
    // SAFETY: SYS_perf_event_open syscall arguments are correct and return value is checked.
    let fd =
        unsafe { libc::syscall(libc::SYS_perf_event_open, &attr, pid, cpu, group, flags) as i32 };
    let event_fd = match fd {
        libc::E2BIG => {
            error!("Too many events or attributes specified.");
            return Err(PerfError::EventOpenError);
        }
        libc::EACCES => {
            error!("Permission denied.");
            return Err(PerfError::EventOpenError);
        }
        libc::EBADF => {
            error!("Invalid group file descriptor.");
            return Err(PerfError::EventOpenError);
        }
        libc::EBUSY => {
            error!("PMU exclusive access already taken.");
            return Err(PerfError::EventOpenError);
        }
        libc::EFAULT => {
            error!("Invalid attribute pointer.");
            return Err(PerfError::EventOpenError);
        }
        libc::EINTR => {
            error!("Mix perf and ftrace handling for a uprobe.");
            return Err(PerfError::EventOpenError);
        }
        libc::EINVAL => {
            error!("Invalid event argument. Consider if sample_freq > max, sample_type out of range,...");
            return Err(PerfError::EventOpenError);
        }
        libc::EMFILE => {
            error!("Too many open file descriptors.");
            return Err(PerfError::EventOpenError);
        }
        libc::ENODEV => {
            error!("Feature not supported on PMU.");
            return Err(PerfError::EventOpenError);
        }
        libc::ENOENT => {
            error!("type_ setting is not valid.");
            return Err(PerfError::EventOpenError);
        }
        libc::ENOSYS => {
            error!("Sample stack user set in sample_type and is not supported on hw.");
            return Err(PerfError::EventOpenError);
        }
        libc::ENOTSUP => {
            error!("Requested feature is not supported.");
            return Err(PerfError::EventOpenError);
        }
        libc::EPERM => {
            error!("Permission denied.");
            return Err(PerfError::EventOpenError);
        }
        _ => fd,
    };
    Ok(event_fd)
}

pub unsafe fn mmap_perf_buffer(
    fd: i32,
    num_pages: usize,
) -> Result<*mut perf_event_mmap_page, PerfError> {
    // SAFETY: Just out here getting page size.
    let page_size = match libc::sysconf(libc::_SC_PAGESIZE) {
        -1 => {
            error!("Failed to get page size.");
            return Err(PerfError::MmapError);
        }
        size => size as usize,
    };
    let mmap_size = page_size * num_pages;
    debug!("Mmap size: {:#02x}", mmap_size);
    // MMAP region must be 1 + 2^n pages. First for header page and then ring buffer.
    if ((num_pages - 1) & (num_pages - 2)) != 0 {
        error!("Number of pages must be 1 + 2^n.");
        return Err(PerfError::MmapError);
    }
    // SAFETY: Caller is responsible for ensuring that the file descriptor is valid.
    match libc::mmap(
        std::ptr::null_mut(),
        mmap_size,
        libc::PROT_READ | libc::PROT_WRITE,
        libc::MAP_SHARED,
        fd,
        0,
    ) {
        libc::MAP_FAILED => {
            error!("Failed to mmap perf buffer.");
            return Err(PerfError::MmapError);
        }
        ptr => Ok(ptr as *mut perf_event_mmap_page),
    }
}

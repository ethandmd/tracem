use std::{
    fs::File,
    mem::zeroed,
    os::fd::{AsRawFd, FromRawFd},
    ptr::copy_nonoverlapping,
};

use log::{debug, error, info};

pub mod perf_sys {
    include!(concat!(env!("OUT_DIR"), "/perf-sys.rs"));
}

use mio::{unix::SourceFd, Events, Interest, Poll, Token};
pub use perf_sys::*;

const MMAP_PAGES: usize = 1 + (1 << 16); // 1MiB

#[derive(Debug)]
pub enum PerfError {
    EventOpen,
    Mmap,
    Poll,
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
    pub fn set_sample_period(&mut self, period: u64) {
        if self.freq() == 1 {
            self.set_freq(0);
        }
        self.__bindgen_anon_1.sample_period = period;
    }
    pub fn get_sample_period(&self) -> u64 {
        unsafe { self.__bindgen_anon_1.sample_period }
    }
    pub fn get_sample_freq(&self) -> u64 {
        unsafe { self.__bindgen_anon_1.sample_freq }
    }
}

pub struct PerfEvent {
    fd: File,
    mmap_hdr: Option<*mut perf_event_mmap_page>,
    mmap_size: usize,
}

impl PerfEvent {
    pub fn new(
        attr: perf_event_attr,
        pid: i32,
        cpu: i32,
        group: Option<&File>,
        flags: i32,
    ) -> Result<Self, PerfError> {
        let group_fd = match group {
            Some(g) => g.as_raw_fd(),
            None => -1,
        };
        debug!(
            "Opening perf event on pid: {}, cpu: {}, with flags: {}",
            pid, cpu, flags
        );
        // SAFETY: SYS_perf_event_open syscall arguments are correct and return value is checked.
        let fd = unsafe {
            libc::syscall(libc::SYS_perf_event_open, &attr, pid, cpu, group_fd, flags) as i32
        };
        if fd < 0 {
            error!("Failed to open perf event.");
            return Err(PerfError::EventOpen);
        }

        // SAFETY: Can only be one owner of the file descriptor.
        let mut event = unsafe {
            Self {
                fd: File::from_raw_fd(fd),
                mmap_hdr: None,
                mmap_size: 0,
            }
        };
        if attr.get_sample_period() != 0 && group.is_none() {
            // SAFETY: fd is valid. If mmap fails, we return an error.
            // On drop, PerfEvent struct will munmap the buffer.
            unsafe {
                let (mmap_hdr, mmap_size) = mmap_perf_buffer(&event.fd, MMAP_PAGES)?;
                event.mmap_hdr = Some(mmap_hdr);
                event.mmap_size = mmap_size;
            }
        }
        if let Some(group_fd) = group {
            // SAFETY: fd is valid. If ioctl fails, we return an error.
            // On drop, PerfEvent struct will close the file descriptor.
            unsafe {
                let ret = libc::ioctl(fd, perf_ioc_SET_OUTPUT as u64, group_fd.as_raw_fd());
                if ret != 0 {
                    error!("Failed to set output group.");
                    return Err(PerfError::EventOpen);
                }
            }
        }
        Ok(event)
    }

    pub fn enable(&self) -> Result<(), PerfError> {
        // SAFETY: fd is valid. If ioctl fails, we return an error.
        let ret = unsafe { libc::ioctl(self.fd.as_raw_fd(), perf_ioc_ENABLE as u64, 0) };
        if ret != 0 {
            error!("Failed to enable perf event.");
            return Err(PerfError::EventOpen);
        }
        Ok(())
    }

    pub fn reset(&self) -> Result<(), PerfError> {
        // SAFETY: fd is valid. If ioctl fails, we return an error.
        let ret = unsafe { libc::ioctl(self.fd.as_raw_fd(), perf_ioc_RESET as u64, 0) };
        if ret != 0 {
            error!("Failed to reset perf event.");
            return Err(PerfError::EventOpen);
        }
        Ok(())
    }

    pub fn get_fd(&self) -> &File {
        &self.fd
    }

    /// Main event loop for reading samples from the perf sample buffer.
    /// Only valid if the perf event was created with a sample period/freq.
    /// Caller provides a function, f, and a record type, record, to be called
    /// in the event loop for SAMPLE_RECORD types *only*. This allows the caller
    /// to define how each sample record is processed.
    /// struct. This function then blindly trusts
    pub fn sample_loop<F, T: super::PolicyTracker>(
        &self,
        f: F,
        record_size: usize,
        tracker: &mut T,
    ) -> Result<(), PerfError>
    where
        F: Fn(*const u8, &mut T),
    {
        if let Some(mmap) = self.mmap_hdr {
            // SAFETY: mmap ptr is valid. Data offset into mmap region is valid.
            let sample_region = {
                let begin = unsafe { mmap.byte_add((*mmap).data_offset as usize) };
                if begin <= mmap {
                    error!("Data offset is invalid: Less than or equal to mmap hdr.");
                    return Err(PerfError::Mmap);
                } else if begin > unsafe { mmap.byte_add(self.mmap_size) } {
                    error!("Data offset is invalid: Greater than mmap size.");
                    return Err(PerfError::Mmap);
                } else {
                    begin
                }
            };
            let mut events = Events::with_capacity(128);
            let TOKEN = Token(0);
            let mut poll = Poll::new().map_err(|_| PerfError::Poll)?;
            poll.registry()
                .register(
                    &mut SourceFd(&self.fd.as_raw_fd()),
                    TOKEN,
                    Interest::READABLE,
                )
                .map_err(|_| PerfError::Poll)?;
            let mut overflows = 0;
            let mut total_events_read = 0;
            loop {
                poll.poll(&mut events, None).map_err(|_| PerfError::Poll)?;
                for event in events.iter() {
                    overflows += 1;
                    if event.token() == TOKEN && event.is_readable() {
                        let mut events_read = 0;
                        let mut record_buf: Vec<u8> = Vec::with_capacity(record_size);
                        loop {
                            // SAFETY: MMAP sample region bounds are valid and ring buffer is within bounds.
                            // If the next sample record is larger than the difference between the
                            // end of the buffer and the tail we read the record in two parts so
                            // that we don't read past the end of the buffer.
                            unsafe {
                                let tail_mod = ((*mmap).data_tail % (*mmap).data_size) as usize;
                                // Check that the next record is within the bounds of the ring.
                                if record_size as u64 > (*mmap).data_head - (*mmap).data_tail {
                                    debug!("Overflows: {}", overflows);
                                    debug!("Events read: {}", events_read);
                                    debug!("Total events read: {}", total_events_read);
                                    break;
                                }
                                // Check the remaining space in the ring buffer in case we need to
                                // wrap.
                                let rem = (*mmap).data_size as usize - tail_mod;
                                let rem = if rem < record_size { rem } else { record_size };
                                // record buf outlives ptr, and rem is always less than or equal to record_size.
                                // all ptr arithmetic is done as byte count, rem is counting bytes.
                                copy_nonoverlapping(
                                    sample_region.byte_add(tail_mod) as *const u8,
                                    record_buf.as_mut_ptr(),
                                    rem,
                                );
                                if rem < record_size {
                                    copy_nonoverlapping(
                                        sample_region as *const u8,
                                        record_buf.as_mut_ptr().byte_add(rem),
                                        record_size - rem,
                                    );
                                }
                                // Check the record type, we only care about SAMPLE_RECORDs for
                                // now.
                                // SAFETY: record_buf ptr is pointing to data that always has
                                // size_of::<perf_event_header>() bytes at the beginning.
                                let hdr: *const perf_event_header = record_buf.as_ptr().cast();
                                if (*hdr).type_ == perf_event_type_PERF_RECORD_SAMPLE {
                                    // Invoke caller provided function with ptr to raw bytes.
                                    // Asking the caller to make an implicit assumption
                                    // about the size and type of the record is not ideal.
                                    // TODO: Consider a more type safe approach.
                                    f(record_buf.as_ptr(), tracker);
                                }
                                events_read += 1;
                                total_events_read += 1;
                                (*mmap).data_tail += record_size as u64;
                            }
                        }
                        tracker.execute();
                    } else if event.is_read_closed() {
                        info!("Read closed.");
                        return Ok(());
                    }
                }
            }
        } else {
            error!("No perf buffer to read samples from.");
            Err(PerfError::Mmap)
        }
    }
}

impl Drop for PerfEvent {
    fn drop(&mut self) {
        if let Some(mmap) = self.mmap_hdr {
            debug!("Unmapping perf buffer...");
            // SAFETY: mmap ptr is valid.
            let ret = unsafe { libc::munmap(mmap as *mut libc::c_void, self.mmap_size) };
            if ret != 0 {
                error!("Failed to munmap perf buffer.");
            } else {
                debug!("Successfully unmapped perf buffer.");
            }
        }
    }
}

unsafe fn mmap_perf_buffer(
    fd: &File,
    num_pages: usize,
) -> Result<(*mut perf_event_mmap_page, usize), PerfError> {
    // SAFETY: Just out here getting page size.
    let page_size = match libc::sysconf(libc::_SC_PAGESIZE) {
        -1 => {
            error!("Failed to get page size.");
            return Err(PerfError::Mmap);
        }
        size => size as usize,
    };
    let mmap_size = page_size * num_pages;
    debug!("Mmap size: {:#02x}", mmap_size);
    // MMAP region must be 1 + 2^n pages. First for header page and then ring buffer.
    if ((num_pages - 1) & (num_pages - 2)) != 0 {
        error!("Number of pages must be 1 + 2^n.");
        return Err(PerfError::Mmap);
    }
    // SAFETY: Caller is responsible for ensuring that the file descriptor is valid.
    match libc::mmap(
        std::ptr::null_mut(),
        mmap_size,
        libc::PROT_READ | libc::PROT_WRITE,
        libc::MAP_SHARED,
        fd.as_raw_fd(),
        0,
    ) {
        libc::MAP_FAILED => {
            error!("Failed to mmap perf buffer.");
            Err(PerfError::Mmap)
        }
        ptr => Ok((ptr as *mut perf_event_mmap_page, mmap_size)),
    }
}

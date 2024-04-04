Intel metrics json here:

https://github.com/intel/perfmon

##### Couldn't generate libpfm bindings until I installed clang, libclang-dev packages.

### Get started again 04/04:
Write some tests in `src/perf.rs` for the builder pattern. Want to check some of the bitfield stuff
and a few of the quasi-mutual exclusion cases for attr fields.

Begin writing PerfEventHandle so we can read some data!

### Get started again 04/03:
In `src/perf.rs` finish builder pattern.

Reference: https://bitbucket.org/ajaustin/hemem/src/sosp-submission/src/pebs.c

And `nvim /usr/include/linux/perf_event.h` for the C API.

And [man perf event open](https://www.man7.org/linux/man-pages/man2/perf_event_open.2.html)

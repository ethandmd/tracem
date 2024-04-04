#include <linux/perf_event.h>
#include <linux/hw_breakpoint.h>

// Bindgen doesn't seem to grab these macro definitions.
// Redefining macros
const int perf_ioc_ENABLE = PERF_EVENT_IOC_ENABLE;
const int perf_ioc_DISABLE = PERF_EVENT_IOC_DISABLE;
const int perf_ioc_REFRESH = PERF_EVENT_IOC_REFRESH;
const int perf_ioc_RESET = PERF_EVENT_IOC_RESET;
const int perf_ioc_PERIOD = PERF_EVENT_IOC_PERIOD;
const int perf_ioc_SET_OUTPUT = PERF_EVENT_IOC_SET_OUTPUT;
const int perf_ioc_SET_FILTER = PERF_EVENT_IOC_SET_FILTER;
const int perf_ioc_ID = PERF_EVENT_IOC_ID;
const int perf_ioc_SET_BPF = PERF_EVENT_IOC_SET_BPF;
const int perf_ioc_PAUSE_OUTPUT = PERF_EVENT_IOC_PAUSE_OUTPUT;
const int perf_ioc_QUERY_BPF = PERF_EVENT_IOC_QUERY_BPF;
const int perf_ioc_MODIFY_ATTRIBUTES = PERF_EVENT_IOC_MODIFY_ATTRIBUTES;

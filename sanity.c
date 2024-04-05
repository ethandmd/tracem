#include <linux/perf_event.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/syscall.h>
#include <unistd.h>
#include <sys/mman.h>
#include <inttypes.h>

#define MMAP_SIZE 1 + (1 << 16) * 4096
#define SAMPLE_FREQ 1000
#define SAMPLE_PERIOD 100

struct perf_sample {
	struct perf_event_header header;
	uint64_t ip;
	uint32_t tid;
	uint64_t time;
	uint64_t addr;
};

static long
perf_event_open(struct perf_event_attr *hw_event, pid_t pid,
                int cpu, int group_fd, unsigned long flags)
{
    int ret;

    ret = syscall(SYS_perf_event_open, hw_event, pid, cpu,
                  group_fd, flags);
    return ret;
}

struct perf_event_mmap_page* map_buffer(int fd, size_t mmap_size) {
	void* base = mmap(NULL, mmap_size, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
	if (base == MAP_FAILED) {
		perror("failed to mmap buffer");
		exit(1);
	}
	return (struct perf_event_mmap_page*)base;
}


int main_loop(struct perf_event_mmap_page* mmap_hdr) {
	printf("ip,tid,time,addr\n");
	for (;;) {
		//if (mmap_hdr->data_tail == mmap_hdr->data_head) {
		//	continue;
		//}
		struct perf_event_header* event_hdr =
			(struct perf_event_header*)((char*)mmap_hdr + mmap_hdr->data_offset + (mmap_hdr->data_tail % (mmap_hdr->data_size)));
		struct perf_sample* sample;
		switch(event_hdr->type) {
			case PERF_RECORD_SAMPLE:
				sample = (struct perf_sample*)event_hdr;
				if (sample->addr != 0) {
					printf("%ld,%d,%ld,%lx\n",
						sample->ip, sample->tid, sample->time, sample->addr & ~(4096 -1));
					//mmap_hdr->data_tail += event_hdr->size;
				}
				break;
			default:
				//printf("unknown event type %d\n", event_hdr->type);
				break;
		}
		mmap_hdr->data_tail += event_hdr->size;
	}
}


int
main(int argc, char **argv)
{
	int pid;
	int cpu = -1;
    int                     fd;
	int						ret;
    struct perf_event_attr  pe;
	struct perf_event_mmap_page* mmap_hdr;
	pid = atoi(argv[1]);
    memset(&pe, 0, sizeof(pe));
    pe.type = PERF_TYPE_RAW;
	pe.config = 0xd1 | (0x20 <<8);
	pe.sample_period = SAMPLE_PERIOD;
	//pe.sample_freq = SAMPLE_FREQ;
	//pe.freq = 1;
    pe.size = sizeof(pe);
    pe.sample_type= PERF_SAMPLE_IP | PERF_SAMPLE_TID | PERF_SAMPLE_TIME | PERF_SAMPLE_ADDR;
    pe.disabled = 0;
    pe.exclude_kernel = 1;
    pe.exclude_hv = 1;
	pe.exclude_callchain_kernel = 1;
	pe.exclude_callchain_user = 1;
	pe.precise_ip = 2;

    fd = perf_event_open(&pe, pid, cpu, -1, 0);
    if (fd == -1) {
       fprintf(stderr, "Error opening leader %llx\n", pe.config);
       exit(EXIT_FAILURE);
    }
	mmap_hdr = map_buffer(fd, MMAP_SIZE);
	
	ret = main_loop(mmap_hdr);

    //ioctl(fd, PERF_EVENT_IOC_RESET, 0);
    //ioctl(fd, PERF_EVENT_IOC_ENABLE, 0);

    //printf("Measuring instruction count for this printf\n");

    ioctl(fd, PERF_EVENT_IOC_DISABLE, 0);
    //read(fd, &count, sizeof(count));

    //printf("Used %lld instructions\n", count);
	munmap(mmap_hdr, MMAP_SIZE);
    close(fd);

	return ret;
}

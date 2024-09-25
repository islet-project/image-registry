static void syscall(long n, long arg1, long arg2, long arg3)
{
	__asm__ volatile (
		"mov x8, %0\n"
		"mov x0, %1\n"
		"mov x1, %2\n"
		"mov x2, %3\n"
		"svc 0\n"
		:
		: "r" (n), "r" (arg1), "r" (arg2), "r" (arg3) : "x0", "x1", "x2", "x8", "memory"
		);
}

struct __kernel_timespec {
	long tv_sec;
	long tv_nsec;
};

int _start()
{
	struct __kernel_timespec req, rem;
	req.tv_sec = 1;
	req.tv_nsec = 0;
	rem.tv_sec = 0;
	rem.tv_nsec = 0;

	for(;;) {
		syscall(0x40, 1, (long)"Example Application\n", 20);
		syscall(0x65, (long)&req, (long)&rem, 0);
	}
}

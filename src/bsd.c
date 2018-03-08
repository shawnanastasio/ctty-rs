#include <stdio.h>
#include <stdlib.h>

#include <unistd.h>

#include <sys/sysctl.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/user.h>

// Platform-specific macros
#ifdef __FreeBSD__
#define struct_kinfo_proc struct kinfo_proc
#define kinfo_ctty(kinfo_proc) (kinfo_proc).ki_tdev
#elif __APPLE__
#define struct_kinfo_proc struct kinfo_proc
#define kinfo_ctty(kinfo_proc) (kinfo_proc).kp_eproc.e_tdev
#endif

uint64_t _get_ctty_dev() {
    int mib[4];
    mib[0] = CTL_KERN;
    mib[1] = KERN_PROC;
    mib[2] = KERN_PROC_PID;
    mib[3] = (int)getpid();

    // Run sysctl
    struct_kinfo_proc kp;
    size_t size = sizeof(struct_kinfo_proc);
    int ret = sysctl(mib, 4, &kp, &size, NULL, 0);
    if (ret == -1) {
        return 0;
    }

    return kinfo_ctty(kp);
}

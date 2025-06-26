#include "capability.h"
#include "../audit/audit.h"
#include "../printk/printk.h"

/*
* Dummy current process

* In a real kernel, this would be thread/process-local
*/
typedef struct {
    uint32_t uid;
    uint32_t pid;
    capability_set_t caps;
} task_t;

// Simulated current task
static task_t current_task = {
    .uid = 0,
    .pid = 1,
    .caps = {{[CAP_CHOWN] = 1, [CAP_KILL] = 1}} // give it some basic caps, implement later
};

void capability_init(void) {
    printk("[capability] Initialized capability subsystem \n");
}

// Public function to check if current task has the capability
bool capable(capability_t cap) {
    if (cap >= CAP_MAX)
        return false;

    bool has_cap = current_task.caps.caps[cap];

    if (!has_cap) {
        audit_log(AUDIT_SECURITY, "Capability check failed", current_task.uid, current_task.pid);
        print("[capability] Denied: pid=%u uid=%u cap=%d\n", current_task.pid, current_task.uid, cap);
    }

    return has_cap;
}

void set_capability(capability_set_t *set, capability_t cap, bool value) {
    if (cap < CAP_MAX)
        set->caps[cap] = value ? 1 : 0;
}

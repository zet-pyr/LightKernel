#ifndef _KERNEL_CAPABILITY_H
#define _KERNEL_CAPABILITY_H

#include <stdint.h>
#include <stdbool.h>

// Define some basic capabilities
typedef enum {
    CAP_CHOWN = 0,
    CAP_DAC_OVERRIDE,
    CAP_KILL,
    CAP_NET_ADMIN,
    CAP_SYS_BOOT,
    CAP_SYS_MODULE,
    CAP_MAX
}

// Represents a process's capability set
typedef struct {
    uint8_t caps[CAP_MAX]; // 0 = no, 1 = has cap
} capability_set_t;

// Checks if current process has a specific capability
bool capable(capability_t cap);

// Sets a capability in a process's capability set
void set_capability(capability_set_t *set, capability_t cap, bool value);

// Initializes capability subsystem
void capability_init(void);

#endif // _KERNEL_CAPABILITY_H
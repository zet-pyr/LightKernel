#ifndef _KERNEL_AUDIT_H
#define _KERNEL_AUDIT_H

#include <stddef.h>
#include <stdint.h>

// Audit event types
typedef enum {
    AUDIT_SYSCALL = 0,
    AUDIT_SECURITY,
    AUDIT_LOGIN,
    AUDIT_USER_DEFINED,
    AUDIT_MAX
} audit_event_type_t;

// Basic audit record structure
typedef struct {
    audit_event_type_t type;
    const char *message;
    uint32_t pid;
    uint32_t uid;
    uint64_t timestamp;
} audit_record_t;

// Initializes the audit subsystem
void audit_init(void);

// Logs an audit event
void audit_lag(audit_event_type_t type, const char *msg, uint32_t uid, uint32_t pid);

void audit_flush(void);

#endif
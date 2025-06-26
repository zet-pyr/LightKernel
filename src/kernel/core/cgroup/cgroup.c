#include "cgroup.h"
#include "../printk/printk.h"
#include "../audit/audit.h"
#include <string.h>
#include <stdlib.h>

static cgroup_t cgroups[MAX_CGROUPS];
static size_t cgroup_count = 0;

void cgroup_init(void) {
    memset(cgroups, 0, sizeof(cgroups));
    cgroup_count = 0;
    printk("[cgroup] Initialized cgroup subsystem\n");
}

// Helper to find a cgroup by name
static int cgroup_find_by_name(const char *name) {
    if (!name) return -1;
    for (size_t i = 0; i < cgroup_count; i++) {
        if (cgroups[i].name && strcmp(cgroups[i].name, name) == 0)
            return (int)i;
    }
    return -1;
}

int cgroup_create(const char *name) {
    if (!name || strlen(name) == 0) {
        printk("[cgroup] Error: invalid cgroup name\n");
        return -1;
    }
    if (cgroup_count >= MAX_CGROUPS) {
        printk("[cgroup] Error: max cgroups reached\n");
        return -1;
    }
    if (cgroup_find_by_name(name) != -1) {
        printk("[cgroup] Error: cgroup name '%s' already exists\n", name);
        return -1;
    }

    cgroup_t *cg = &cgroups[cgroup_count];
    cg->id = (uint32_t)cgroup_count;
    cg->name = strdup(name); // Allocate and copy name
    if (!cg->name) {
        printk("[cgroup] Error: failed to allocate name\n");
        return -1;
    }
    cg->task_count = 0;

    audit_log(AUDIT_USER_DEFINED, "Created cgroup", cg->id, 0);
    printk("[cgroup] Created cgroup id=%u name=%s\n", cg->id, name);

    return (int)cgroup_count++;
}

int cgroup_destroy(uint32_t id) {
    if (id >= cgroup_count) {
        printk("[cgroup] Error: invalid cgroup id %u\n", id);
        return -1;
    }

    // Free name memory
    if (cgroups[id].name) {
        free((void*)cgroups[id].name);
        cgroups[id].name = NULL;
    }
    cgroups[id].task_count = 0;

    // Compact the array to remove the destroyed cgroup
    for (size_t i = id; i < cgroup_count - 1; i++) {
        cgroups[i] = cgroups[i + 1];
        cgroups[i].id = (uint32_t)i; // Update id
    }
    memset(&cgroups[cgroup_count - 1], 0, sizeof(cgroup_t));
    cgroup_count--;

    audit_log(AUDIT_USER_DEFINED, "Destroyed cgroup", id, 0);
    printk("[cgroup] Destroyed cgroup id=%u\n", id);

    return 0;
}

int cgroup_attach_task(uint32_t cgroup_id, uint32_t pid) {
    if (cgroup_id >= cgroup_count) {
        printk("[cgroup] Error: invalid cgroup id %u\n", cgroup_id);
        return -1;
    }

    cgroup_t *cg = &cgroups[cgroup_id];

    // Prevent duplicate task
    for (size_t i = 0; i < cg->task_count; i++) {
        if (cg->tasks[i].pid == pid) {
            printk("[cgroup] Error: pid=%u already in cgroup id=%u\n", pid, cgroup_id);
            return -1;
        }
    }

    if (cg->task_count >= MAX_TASKS_PER_CGROUP) {
        printk("[cgroup] Error: cgroup id=%u full\n", cgroup_id);
        return -1;
    }

    cg->tasks[cg->task_count++].pid = pid;
    printk("[cgroup] Attached pid=%u to cgroup id=%u (%s)\n", pid, cgroup_id, cg->name ? cg->name : "(null)");

    audit_log(AUDIT_USER_DEFINED, "Attached task to cgroup", cgroup_id, pid);

    return 0;
}

int cgroup_detach_task(uint32_t cgroup_id, uint32_t pid) {
    if (cgroup_id >= cgroup_count) {
        printk("[cgroup] Error: invalid cgroup id %u\n", cgroup_id);
        return -1;
    }
    cgroup_t *cg = &cgroups[cgroup_id];
    size_t idx = MAX_TASKS_PER_CGROUP;
    for (size_t i = 0; i < cg->task_count; i++) {
        if (cg->tasks[i].pid == pid) {
            idx = i;
            break;
        }
    }
    if (idx == MAX_TASKS_PER_CGROUP) {
        printk("[cgroup] Error: pid=%u not found in cgroup id=%u\n", pid, cgroup_id);
        return -1;
    }
    // Remove task by shifting
    for (size_t i = idx; i < cg->task_count - 1; i++) {
        cg->tasks[i] = cg->tasks[i + 1];
    }
    cg->task_count--;
    printk("[cgroup] Detached pid=%u from cgroup id=%u (%s)\n", pid, cgroup_id, cg->name ? cg->name : "(null)");

    audit_log(AUDIT_USER_DEFINED, "Detached task from cgroup", cgroup_id, pid);

    return 0;
}

void cgroup_dump(void) {
    printk("[cgroup] Dumping all cgroups:\n");

    for (size_t i = 0; i < cgroup_count; i++) {
        cgroup_t *cg = &cgroups[i];
        printk("  [%u] %s: %zu tasks\n", cg->id, cg->name ? cg->name : "(null)", cg->task_count);
        for (size_t j = 0; j < cg->task_count; j++) {
            printk("    - pid: %u\n", cg->tasks[j].pid);
        }
    }
}

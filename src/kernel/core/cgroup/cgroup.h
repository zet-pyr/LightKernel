#ifndef _KERNEL_CGROUP_H
#define _KERNEL_CGROUP_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#define CGROUP_MAX_NAME_LEN 64
#define CGROUP_MAX_TASKS    128
#define CGROUP_MAX_CHILDREN 16

typedef enum {
    CGROUP_OK = 0,
    CGROUP_ERR_NO_SPACE,
    CGROUP_ERR_DUPLICATE,
    CGROUP_ERR_NOT_FOUND,
    CGROUP_ERR_INVALID,
    CGROUP_ERR_FULL,
    // ...
} cgroup_err_t;

typedef struct cgroup {
    char name[CGROUP_MAX_NAME_LEN];
    struct cgroup *parent;
    struct cgroup *children[CGROUP_MAX_CHILDREN];
    size_t child_count;

    uint32_t tasks[CGROUP_MAX_TASKS];
    size_t task_count;
} cgroup_t;

/**
 * Initialize the cgroup subsystem.
 */
void cgroup_init(void);

/**
 * Create a new cgroup with the given name.
 * @param name The name of the cgroup.
 * @param out_cgroup Pointer to store the created cgroup.
 * @return CGROUP_OK on success, error code otherwise.
 */
cgroup_err_t cgroup_create(const char *name, cgroup_t **out_cgroup);

/**
 * Destroy a cgroup and detach all tasks.
 * @param cgroup The cgroup to destroy.
 * @return CGROUP_OK on success, error code otherwise.
 */
cgroup_err_t cgroup_destroy(cgroup_t *cgroup);

/**
 * Attach a task (by pid) to a cgroup.
 * @param cgroup The cgroup.
 * @param pid The process ID to attach.
 * @return CGROUP_OK on success, error code otherwise.
 */
cgroup_err_t cgroup_attach_task(cgroup_t *cgroup, uint32_t pid);

/**
 * Detach a task (by pid) from a cgroup.
 * @param cgroup The cgroup.
 * @param pid The process ID to detach.
 * @return CGROUP_OK on success, error code otherwise.
 */
cgroup_err_t cgroup_detach_task(cgroup_t *cgroup, uint32_t pid);

/**
 * Print all cgroups and their tasks for debugging.
 */
void cgroup_dump(void);

/**
 * Find a cgroup by name.
 * @param name The name to search for.
 * @return Pointer to cgroup or NULL if not found.
 */
cgroup_t *cgroup_find(const char *name);

/**
 * Check if a task is attached to a cgroup.
 * @param cgroup The cgroup.
 * @param pid The process ID.
 * @return true if attached, false otherwise.
 */
bool cgroup_has_task(const cgroup_t *cgroup, uint32_t pid);

#endif

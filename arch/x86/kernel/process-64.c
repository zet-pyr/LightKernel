// Fichier des processus 64-bit sans Linux

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <signal.h>
#include <errno.h>
#include <time.h>
#include "process.h"
#include <sys/syscall.h>

// 1. Define the syscall number for process management
#define SYS_CREATE_PROCESS 355
#define SYS_WAIT_PROCESS 356
#define SYS_KILL_PROCESS 357

// 2. Structure to store process information
struct process_info {
    pid_t pid;
    time_t start_time;
    int status;
};

// 3. Function to create a process with advanced management
pid_t create_process(void (*func)(void *), void *arg, struct process_info *info) {
    pid_t pid = syscall(SYS_CREATE_PROCESS, func, arg);
    if (pid < 0) {
        perror("Create process failed");
        return -1;
    }
    // Parent: store process info
    if (info) {
        info->pid = pid;
        info->start_time = time(NULL);
        info->status = 0;
    }
    return pid;
}

// 4. Function to wait for a process to finish and update info
int wait_process(struct process_info *info) {
    int status;
    if (syscall(SYS_WAIT_PROCESS, info->pid, &status) == -1) {
        perror("Wait process failed");
        return -1;
    }
    info->status = status;
    if (WIFEXITED(status)) {
        return WEXITSTATUS(status);
    } else if (WIFSIGNALED(status)) {
        fprintf(stderr, "Process terminated by signal %d\n", WTERMSIG(status));
        return -1;
    }
    return 0;
}

// 5. Function to kill a process and update info
int kill_process(struct process_info *info, int sig) {
    if (syscall(SYS_KILL_PROCESS, info->pid, sig) == -1) {
        perror("Kill process failed");
        return -1;
    }
    info->status = -1; // Indicate that the process was killed
    return 0;
}
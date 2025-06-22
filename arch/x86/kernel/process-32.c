// Fichier pour Processus 32-bit sans linux

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <signal.h>

// Structure pour stocker des informations sur un processus
struct process_info {
    pid_t pid;
    time_t start_time;
    int status;
};

// Fonction pour créer un processus avec gestion avancée
pid_t create_process(void (*func)(void *), void *arg, struct process_info *info) {
    pid_t pid = fork();
    if (pid < 0) {
        perror("Fork failed");
        return -1;
    } else if (pid == 0) {
        func(arg);
        exit(EXIT_SUCCESS);
    }
    // Parent : stocke les infos du processus
    if (info) {
        info->pid = pid;
        info->start_time = time(NULL);
        info->status = 0;
    }
    return pid;
}

// Fonction pour attendre la fin d'un processus et mettre à jour info
int wait_process(struct process_info *info) {
    int status;
    if (waitpid(info->pid, &status, 0) == -1) {
        perror("waitpid failed");
        return -1;
    }
    info->status = status;
    if (WIFEXITED (status)) {
        return WEXITSTATUS(status);
    } else if (WIFSIGNALED(status)) {
        fprintf(stderr, "Process terminated by signal %d\n", WTERMSIG(status));
        return -1;
    }
    return 0;
}

// Fonction pour terminer un processus et mettre à jour info
int kill_process(struct process_info *info, int sig) {
    if (kill(info->pid, sig) == -1) {
        perror("kill failed");
        return -1;
    } else {
        info->status = -1; // Indique que le processus a été tué
    }
    return 0;
}
// Fichier de gestion avancée des processus x86 sans Linux

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
        perror("The fork failed");
        return -1;
    } else if (pid == 0) {
        // Enfant : exécute la fonction passée
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
    if (WIFEXITED(status)) {
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
    }
    info->status = -1;
    return 0;
}

// Fonction pour afficher des informations détaillées sur un processus
void print_process_info(const struct process_info *info) {
    printf("Process PID: %d\n", info->pid);
    printf("Start time: %s", ctime(&info->start_time));
    printf("Status: %d\n", info->status);
    // Pour plus d'infos, lire /proc/[pid] si disponible
}

// Fonction pour envoyer un signal personnalisé à un processus
int send_signal(struct process_info *info, int sig) {
    if (kill(info->pid, sig) == -1) {
        perror("send_signal failed");
        return -1;
    }
    return 0;
}
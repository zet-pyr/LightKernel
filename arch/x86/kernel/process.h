// Fichier pour le header/config des processus

#ifndef PROCESS_H
#define PROCESS_H

#include <sys/types.h>
#include <time.h>
#include <signal.h>

#ifdef __cplusplus
extern "C" {
#endif

// Structure pour stocker des informations sur un processus
struct process_info {
    pid_t pid;
    time_t start_time;
    int status;
};

// Crée un processus et remplit la structure info
pid_t create_process(void (*func)(void *), void *arg, struct process_info *info);

// Attend la fin d'un processus et met à jour info->status
int wait_process(struct process_info *info);

// Termine un processus avec le signal donné et met à jour info->status
int kill_process(struct process_info *info, int sig);

// Affiche les informations détaillées sur un processus
void print_process_info(const struct process_info *info);

// Envoie un signal personnalisé à un processus
int send_signal(struct process_info *info, int sig);

#ifdef __cplusplus
}
#endif

#endif // PROCESS_H
// Fichier header pour les events du kernel
#ifndef KERNEL_CORE_EVENTS_EVENT_H
#define KERNEL_CORE_EVENTS_EVENT_H

namespace Kernel {
    class Event {
        public:
            // Constructeur
            Event(int id, const char* name) : id(id), name(name) {}

            // Destructeur
            virtual ~Event() {}

            // Méthode pour obtenir l'ID de l'event
            int getId() const {
                return id;
            }
            // Méthode pour obtenir le nom de l'event
            const char* getName() const {
                return name;
            }

            // Méthode pour exécuter l'event
            virtual void execute() = 0;

        protected:
            int id;
            const char* name;
    };

    // Enumération pour les types d'events
    enum EventType {
        EVENT_TYPE_GENERIC,
        EVENT_TYPE_SYSTEM,
        EVENT_TYPE_USER,
        EVENT_TYPE_NETWORK,
        EVENT_TYPE_FILE,
        EVENT_TYPE_CUSTOM
    };
    // Enumération pour les statuts des events
    enum EventStatus {
        EVENT_STATUS_PENDING,
        EVENT_STATUS_IN_PROGRESS,
        EVENT_STATUS_COMPLETED,
        EVENT_STATUS_FAILED,
        EVENT_STATUS_CANCELLED
    };

    // Classe pour les events personnalisés
    class CustomEvent : public Event {
        public:
            // Constructeur
            CustomEvent(int id, const char* name, void (*callback)())
                : Event(id, name), callback(callback) {}
            
            // Destructeur
            virtual ~CustomEvent() {}

            // Méthode pour exécuter l'event
            void execute() override {
                if (callback) {
                    callback(); // Appel de la fonction de rappel
                }
            }
        private:
            void (*callback)(); // Pointeur vers la fonction de rappel
    };
    // Classe pour les events système
    class SystemEvent : public Event {
        public:
            // Constructeur
            SystemEvent(int id, const char* name, const char* systemInfo)
                : Event(id, name), systemInfo(systemInfo) {}

            // Destructeur
            virtual ~SystemEvent() {}

            // Méthode pour exécuter l'event
            void execute() override {
                // Logique spécifique à l'event système
            }

            // Méthode pour obtenir les informations système
            const char* getSystemInfo() const {
                return systemInfo;
            }

        private:
            const char* systemInfo; // Informations spécifiques au système
    };

    // Classe pour les events réseau
    class NetworkEvent : public Event {
        public:
            // Constructeur
            NetworkEvent(int id, const char* name, const char* networkInfo)
                : Event(id, name), networkInfo(networkInfo) {}

            // Destructeur
            virtual ~NetworkEvent() {}

            // Méthode pour exécuter l'event
            void execute() override {
                // Logique spécifique à l'event réseau
            }

            // Méthode pour obtenir les informations réseau
            const char* getNetworkInfo() const {
                return networkInfo;
            }

        private:
            const char* networkInfo; // Informations spécifiques au réseau
    };

    // Classe pour les events de fichier
    class FileEvent : public Event {
        public:
            // Constructeur
            FileEvent(int id, const char* name, const char* filePath)
                : Event(id, name), filePath(filePath) {}

            // Destructeur
            virtual ~FileEvent() {}

            // Méthode pour exécuter l'event
            void execute() override {
                // Logique spécifique à l'event fichier
            }

            // Méthode pour obtenir le chemin du fichier
            const char* getFilePath() const {
                return filePath;
            }

        private:
            const char* filePath; // Chemin du fichier concerné
    };

} // namespace Kernel

#endif // KERNEL_CORE_EVENTS_EVENT_H
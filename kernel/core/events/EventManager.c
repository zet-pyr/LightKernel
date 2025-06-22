// Fichier de l'event manager qui gère les events du kernel
#include "EventManager.h"
#include "Event.h"
#include "EventQueue.h"
#include "EventListener.h"
#include "EventListenerList.h"
#include "EventManagerPrivate.h"
#include "EventManagerPublic.h"
#include "EventManagerConfig.h"
#include "EventManagerDebug.h"

// Constructeur de l'EventManager
EventManager::EventManager() {
    this->eventQueue = new EventQueue();
    this->eventListenerList = new EventListenerList();
    this->eventManagerPrivate = new EventManagerPrivate();
}

// Destructeur de l'EventManager
EventManager::~EventManager() {
    delete this->eventQueue;
    delete this->eventListenerList;
    delete this->eventManagerPrivate;
}

typedef EventManager* EventManagerPtr;

typedef enum {
    EVENT_MANAGER_SUCCESS = 0,
    EVENT_MANAGER_WARNING = 1,
    EVENT_MANAGER_ERROR = 2,
    EVENT_MANAGER_CRITICAL = 3
} EventManagerStatus;

// Méthode pour ajouter un listener à l'EventManager
EventManagerStatus EventManager::addListener(EventListener* listener) {
    if (listener == nullptr) {
        return EVENT_MANAGER_ERROR; // Listener is null
    } else if (this->eventListenerList->contains(listener)) {
        return EVENT_MANAGER_WARNING; // Listener already exists
    } else {
        this->eventListenerList->add(listener);
        return EVENT_MANAGER_SUCCESS; // Listener added successfully
    }
}

// Méthode pour supprimer un listener de l'EventManager
EventManagerStatus EventManager::removeListener(EventListener* listener) {
    if (listener == nullptr) {
        return EVENT_MANAGER_ERROR; // Listener is null
    } else if (!this->eventListenerList->contains(listener)) {
        return EVENT_MANAGER_WARNING; // Listener does not exist
    } else {
        this->eventListenerList->remove(listener);
        return EVENT_MANAGER_SUCCESS; // Listener removed successfully
    }
}

// Méthode pour publier un event dans l'EventManager
EventManagerStatus EventManager::publishEvent(Event* event) {
    if (event == nullptr) {
        return EVENT_MANAGER_ERROR; // Event is null
    } else {
        this->eventQueue->enqueue(event);
        this->eventManagerPrivate->notifyListeners(event);
        return EVENT_MANAGER_SUCCESS; // Event published successfully
    }
}

// Méthode pour traiter les events dans l'EventManager
EventManagerStatus EventManager::processEvents() {
    while (!this->eventQueue->isEmpty()) {
        Event* event = this->eventQueue->dequeue();
        if (event == nullptr) {
            return EVENT_MANAGER_ERROR; // Failed to dequeue event
        } else {
            this->eventManagerPrivate->notifyListeners(event);
            delete event;
        }
    }

    return EVENT_MANAGER_SUCCESS; // All events processed successfully
}

// Méthode pour obtenir la liste des listeners de l'EventManager
EventListenerList* EventManager::getListeners() {
    return this->eventListenerList; // Return the list of listeners
}

// Méthode pour obtenir la file d'attente des events de l'EventManager
EventQueue* EventManager::getEventQueue() {
    return this->eventQueue; // Return the event queue
}

// Méthode pour obtenir les informations privées de l'EventManager
EventManagerPrivate* EventManager::getPrivateInfo() {
    return this->eventManagerPrivate; // Return the private information
}

// Méthode pour obtenir le statut de l'EventManager
EventManagerStatus EventManager::getStatus() {
    return EVENT_MANAGER_SUCCESS; // Return the status of the EventManager
}

// Méthode pour réinitialiser l'EventManager
EventManagerStatus EventManager::reset() {
    this->eventQueue->clear();
    this->eventListenerList->clear();
    this->eventManagerPrivate->reset();
    return EVENT_MANAGER_SUCCESS; // EventManager reset successfully
}

// Méthode pour obtenir le nombre d'events dans la file d'attente de l'EventManager
int EventManager::getEventCount() {
    return this->eventQueue->size();
}

// Méthode pour obtenir le nombre de listeners de l'EventManager
int EventManager::getListenerCount() {
    return this->eventListenerList->size();
}

// Méthode pour vérifier si l'EventManager est vide
bool EventManager::isEmpty() {
    return this->eventQueue->isEmpty() && this->eventListenerList->isEmpty();
}

// Méthode pour vérifier si l'EventManager est actif
bool EventManager::isActive() {
    return !this->eventQueue->isEmpty() || !this->eventListenerList->isEmpty();
}

// Méthode pour obtenir le nom de l'EventManager
const char* EventManager::getName() {
    return "EventManager"; // Return the name of the EventManager
}
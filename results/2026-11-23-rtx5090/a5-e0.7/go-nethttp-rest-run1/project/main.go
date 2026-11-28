package main

import (
	"encoding/json"
	"log"
	"net/http"
	"sort"
	"strconv"
	"sync"
)

type Task struct {
	ID    uint64 `json:"id"`
	Title string `json:"title"`
	Done  bool   `json:"done"`
}

type app struct {
	mu    sync.RWMutex
	tasks map[uint64]Task
	nextID uint64
}

func App() http.Handler {
	a := &app{
		tasks:  make(map[uint64]Task),
		nextID: 1,
	}

	mux := http.NewServeMux()
	mux.HandleFunc("GET /health", a.handleHealth)
	mux.HandleFunc("GET /tasks", a.handleListTasks)
	mux.HandleFunc("POST /tasks", a.handleCreateTask)
	mux.HandleFunc("GET /tasks/{id}", a.handleGetTask)
	mux.HandleFunc("PUT /tasks/{id}", a.handleUpdateTask)
	mux.HandleFunc("DELETE /tasks/{id}", a.handleDeleteTask)
	return mux
}

func (a *app) handleHealth(w http.ResponseWriter, r *http.Request) {
	writeJSON(w, http.StatusOK, map[string]string{"status": "ok"})
}

func (a *app) handleListTasks(w http.ResponseWriter, r *http.Request) {
	a.mu.RLock()
	defer a.mu.RUnlock()

	list := make([]Task, 0, len(a.tasks))
	for _, t := range a.tasks {
		list = append(list, t)
	}
	sort.Slice(list, func(i, j int) bool { return list[i].ID < list[j].ID })

	writeJSON(w, http.StatusOK, list)
}

func (a *app) handleCreateTask(w http.ResponseWriter, r *http.Request) {
	var req struct {
		Title string `json:"title"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "invalid JSON body")
		return
	}

	a.mu.Lock()
	defer a.mu.Unlock()

	t := Task{
		ID:    a.nextID,
		Title: req.Title,
		Done:  false,
	}
	a.tasks[t.ID] = t
	a.nextID++

	writeJSON(w, http.StatusCreated, t)
}

func (a *app) handleGetTask(w http.ResponseWriter, r *http.Request) {
	id, ok := parseTaskID(w, r)
	if !ok {
		return
	}

	a.mu.RLock()
	defer a.mu.RUnlock()

	t, exists := a.tasks[id]
	if !exists {
		writeError(w, http.StatusNotFound, "task not found")
		return
	}

	writeJSON(w, http.StatusOK, t)
}

func (a *app) handleUpdateTask(w http.ResponseWriter, r *http.Request) {
	id, ok := parseTaskID(w, r)
	if !ok {
		return
	}

	var req struct {
		Title string `json:"title"`
		Done  bool   `json:"done"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "invalid JSON body")
		return
	}

	a.mu.Lock()
	defer a.mu.Unlock()

	t, exists := a.tasks[id]
	if !exists {
		writeError(w, http.StatusNotFound, "task not found")
		return
	}

	t.Title = req.Title
	t.Done = req.Done
	a.tasks[id] = t

	writeJSON(w, http.StatusOK, t)
}

func (a *app) handleDeleteTask(w http.ResponseWriter, r *http.Request) {
	id, ok := parseTaskID(w, r)
	if !ok {
		return
	}

	a.mu.Lock()
	defer a.mu.Unlock()

	if _, exists := a.tasks[id]; !exists {
		writeError(w, http.StatusNotFound, "task not found")
		return
	}

	delete(a.tasks, id)
	w.WriteHeader(http.StatusNoContent)
}

func parseTaskID(w http.ResponseWriter, r *http.Request) (uint64, bool) {
	id, err := strconv.ParseUint(r.PathValue("id"), 10, 64)
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid task id")
		return 0, false
	}
	return id, true
}

func writeJSON(w http.ResponseWriter, status int, v interface{}) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(v)
}

func writeError(w http.ResponseWriter, status int, msg string) {
	writeJSON(w, status, map[string]string{"error": msg})
}

func main() {
	log.Fatal(http.ListenAndServe(":3000", App()))
}

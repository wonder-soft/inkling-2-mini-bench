package main

import (
	"encoding/json"
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
	next  uint64
}

func App() http.Handler {
	a := &app{
		tasks: make(map[uint64]Task),
		next:  1,
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
	tasks := make([]Task, 0, len(a.tasks))
	for _, t := range a.tasks {
		tasks = append(tasks, t)
	}
	sort.Slice(tasks, func(i, j int) bool { return tasks[i].ID < tasks[j].ID })
	writeJSON(w, http.StatusOK, tasks)
}

func (a *app) handleCreateTask(w http.ResponseWriter, r *http.Request) {
	var body struct {
		Title string `json:"title"`
	}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		writeError(w, http.StatusBadRequest, "invalid JSON body")
		return
	}
	if body.Title == "" {
		writeError(w, http.StatusBadRequest, "title is required")
		return
	}
	a.mu.Lock()
	defer a.mu.Unlock()
	t := Task{ID: a.next, Title: body.Title, Done: false}
	a.next++
	a.tasks[t.ID] = t
	writeJSON(w, http.StatusCreated, t)
}

func (a *app) handleGetTask(w http.ResponseWriter, r *http.Request) {
	id, ok := parseID(w, r)
	if !ok {
		return
	}
	a.mu.RLock()
	defer a.mu.RUnlock()
	t, found := a.tasks[id]
	if !found {
		writeError(w, http.StatusNotFound, "task not found")
		return
	}
	writeJSON(w, http.StatusOK, t)
}

func (a *app) handleUpdateTask(w http.ResponseWriter, r *http.Request) {
	id, ok := parseID(w, r)
	if !ok {
		return
	}
	var body struct {
		Title string `json:"title"`
		Done  bool   `json:"done"`
	}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		writeError(w, http.StatusBadRequest, "invalid JSON body")
		return
	}
	if body.Title == "" {
		writeError(w, http.StatusBadRequest, "title is required")
		return
	}
	a.mu.Lock()
	defer a.mu.Unlock()
	t, found := a.tasks[id]
	if !found {
		writeError(w, http.StatusNotFound, "task not found")
		return
	}
	t.Title = body.Title
	t.Done = body.Done
	a.tasks[id] = t
	writeJSON(w, http.StatusOK, t)
}

func (a *app) handleDeleteTask(w http.ResponseWriter, r *http.Request) {
	id, ok := parseID(w, r)
	if !ok {
		return
	}
	a.mu.Lock()
	defer a.mu.Unlock()
	if _, found := a.tasks[id]; !found {
		writeError(w, http.StatusNotFound, "task not found")
		return
	}
	delete(a.tasks, id)
	w.WriteHeader(http.StatusNoContent)
}

func parseID(w http.ResponseWriter, r *http.Request) (uint64, bool) {
	id, err := strconv.ParseUint(r.PathValue("id"), 10, 64)
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid task id")
		return 0, false
	}
	return id, true
}

func writeJSON(w http.ResponseWriter, status int, v any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(v)
}

func writeError(w http.ResponseWriter, status int, msg string) {
	writeJSON(w, status, map[string]string{"error": msg})
}

func main() {
	_ = http.ListenAndServe(":3000", App())
}

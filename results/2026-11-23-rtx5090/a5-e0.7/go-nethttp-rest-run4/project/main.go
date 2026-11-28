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

type server struct {
	mu    sync.RWMutex
	tasks map[uint64]Task
	nextID uint64
}

func App() http.Handler {
	s := &server{
		tasks:  make(map[uint64]Task),
		nextID: 1,
	}
	mux := http.NewServeMux()
	mux.HandleFunc("GET /health", s.handleHealth)
	mux.HandleFunc("GET /tasks", s.handleListTasks)
	mux.HandleFunc("POST /tasks", s.handleCreateTask)
	mux.HandleFunc("GET /tasks/{id}", s.handleGetTask)
	mux.HandleFunc("PUT /tasks/{id}", s.handleUpdateTask)
	mux.HandleFunc("DELETE /tasks/{id}", s.handleDeleteTask)
	return mux
}

func (s *server) handleHealth(w http.ResponseWriter, r *http.Request) {
	writeJSON(w, http.StatusOK, map[string]string{"status": "ok"})
}

func (s *server) handleListTasks(w http.ResponseWriter, r *http.Request) {
	s.mu.RLock()
	tasks := make([]Task, 0, len(s.tasks))
	for _, t := range s.tasks {
		tasks = append(tasks, t)
	}
	s.mu.RUnlock()
	sort.Slice(tasks, func(i, j int) bool { return tasks[i].ID < tasks[j].ID })
	writeJSON(w, http.StatusOK, tasks)
}

func (s *server) handleCreateTask(w http.ResponseWriter, r *http.Request) {
	var body struct {
		Title string `json:"title"`
	}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		writeError(w, http.StatusBadRequest, "invalid JSON body")
		return
	}
	s.mu.Lock()
	task := Task{ID: s.nextID, Title: body.Title, Done: false}
	s.nextID++
	s.tasks[task.ID] = task
	s.mu.Unlock()
	writeJSON(w, http.StatusCreated, task)
}

func (s *server) handleGetTask(w http.ResponseWriter, r *http.Request) {
	id, ok := parseID(r)
	if !ok {
		writeError(w, http.StatusNotFound, "task not found")
		return
	}
	s.mu.RLock()
	task, found := s.tasks[id]
	s.mu.RUnlock()
	if !found {
		writeError(w, http.StatusNotFound, "task not found")
		return
	}
	writeJSON(w, http.StatusOK, task)
}

func (s *server) handleUpdateTask(w http.ResponseWriter, r *http.Request) {
	id, ok := parseID(r)
	if !ok {
		writeError(w, http.StatusNotFound, "task not found")
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
	s.mu.Lock()
	task, found := s.tasks[id]
	if !found {
		s.mu.Unlock()
		writeError(w, http.StatusNotFound, "task not found")
		return
	}
	task.Title = body.Title
	task.Done = body.Done
	s.tasks[id] = task
	s.mu.Unlock()
	writeJSON(w, http.StatusOK, task)
}

func (s *server) handleDeleteTask(w http.ResponseWriter, r *http.Request) {
	id, ok := parseID(r)
	if !ok {
		writeError(w, http.StatusNotFound, "task not found")
		return
	}
	s.mu.Lock()
	_, found := s.tasks[id]
	if found {
		delete(s.tasks, id)
	}
	s.mu.Unlock()
	if !found {
		writeError(w, http.StatusNotFound, "task not found")
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func parseID(r *http.Request) (uint64, bool) {
	id, err := strconv.ParseUint(r.PathValue("id"), 10, 64)
	if err != nil {
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

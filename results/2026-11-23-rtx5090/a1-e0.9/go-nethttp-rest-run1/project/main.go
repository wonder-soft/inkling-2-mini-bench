package main

import (
	"encoding/json"
	"log"
	"net/http"
	"sort"
	"strconv"
	"strings"
	"sync"
)

type Task struct {
	ID    uint64 `json:"id"`
	Title string `json:"title"`
	Done  bool   `json:"done"`
}

type store struct {
	mu     sync.RWMutex
	tasks  map[uint64]Task
	nextID uint64
}

func newStore() *store {
	return &store{
		tasks:  make(map[uint64]Task),
		nextID: 1,
	}
}

func App() http.Handler {
	s := newStore()
	mux := http.NewServeMux()
	mux.HandleFunc("GET /health", healthHandler)
	mux.HandleFunc("GET /tasks", s.listTasks)
	mux.HandleFunc("POST /tasks", s.createTask)
	mux.HandleFunc("GET /tasks/{id}", s.getTask)
	mux.HandleFunc("PUT /tasks/{id}", s.updateTask)
	mux.HandleFunc("DELETE /tasks/{id}", s.deleteTask)
	return mux
}

func healthHandler(w http.ResponseWriter, r *http.Request) {
	writeJSON(w, http.StatusOK, map[string]string{"status": "ok"})
}

func (s *store) listTasks(w http.ResponseWriter, r *http.Request) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	tasks := make([]Task, 0, len(s.tasks))
	for _, task := range s.tasks {
		tasks = append(tasks, task)
	}
	sort.Slice(tasks, func(i, j int) bool { return tasks[i].ID < tasks[j].ID })

	writeJSON(w, http.StatusOK, tasks)
}

func (s *store) createTask(w http.ResponseWriter, r *http.Request) {
	var req struct {
		Title string `json:"title"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "invalid JSON body")
		return
	}
	title := strings.TrimSpace(req.Title)
	if title == "" {
		writeError(w, http.StatusBadRequest, "title is required")
		return
	}

	s.mu.Lock()
	defer s.mu.Unlock()

	task := Task{ID: s.nextID, Title: title, Done: false}
	s.tasks[task.ID] = task
	s.nextID++

	writeJSON(w, http.StatusCreated, task)
}

func (s *store) getTask(w http.ResponseWriter, r *http.Request) {
	id, err := taskIDFromRequest(r)
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid task id")
		return
	}

	s.mu.RLock()
	defer s.mu.RUnlock()

	task, found := s.tasks[id]
	if !found {
		writeError(w, http.StatusNotFound, "task not found")
		return
	}
	writeJSON(w, http.StatusOK, task)
}

func (s *store) updateTask(w http.ResponseWriter, r *http.Request) {
	id, err := taskIDFromRequest(r)
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid task id")
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
	title := strings.TrimSpace(req.Title)
	if title == "" {
		writeError(w, http.StatusBadRequest, "title is required")
		return
	}

	s.mu.Lock()
	defer s.mu.Unlock()

	task, found := s.tasks[id]
	if !found {
		writeError(w, http.StatusNotFound, "task not found")
		return
	}
	task.Title = title
	task.Done = req.Done
	s.tasks[id] = task

	writeJSON(w, http.StatusOK, task)
}

func (s *store) deleteTask(w http.ResponseWriter, r *http.Request) {
	id, err := taskIDFromRequest(r)
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid task id")
		return
	}

	s.mu.Lock()
	defer s.mu.Unlock()

	if _, found := s.tasks[id]; !found {
		writeError(w, http.StatusNotFound, "task not found")
		return
	}
	delete(s.tasks, id)

	w.WriteHeader(http.StatusNoContent)
}

func taskIDFromRequest(r *http.Request) (uint64, error) {
	raw := r.PathValue("id")
	return strconv.ParseUint(raw, 10, 64)
}

func writeJSON(w http.ResponseWriter, status int, v any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	json.NewEncoder(w).Encode(v)
}

func writeError(w http.ResponseWriter, status int, message string) {
	writeJSON(w, status, map[string]string{"error": message})
}

func main() {
	log.Fatal(http.ListenAndServe(":3000", App()))
}

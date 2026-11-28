package main

import (
	"encoding/json"
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

type taskStore struct {
	mu    sync.RWMutex
	tasks map[uint64]Task
	next  uint64
}

func newTaskStore() *taskStore {
	return &taskStore{
		tasks: make(map[uint64]Task),
		next:  1,
	}
}

func App() http.Handler {
	store := newTaskStore()
	mux := http.NewServeMux()

	mux.HandleFunc("GET /health", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, map[string]string{"status": "ok"})
	})

	mux.HandleFunc("GET /tasks", func(w http.ResponseWriter, r *http.Request) {
		store.listTasks(w)
	})

	mux.HandleFunc("POST /tasks", func(w http.ResponseWriter, r *http.Request) {
		store.createTask(w, r)
	})

	mux.HandleFunc("GET /tasks/{id}", func(w http.ResponseWriter, r *http.Request) {
		store.getTask(w, r)
	})

	mux.HandleFunc("PUT /tasks/{id}", func(w http.ResponseWriter, r *http.Request) {
		store.updateTask(w, r)
	})

	mux.HandleFunc("DELETE /tasks/{id}", func(w http.ResponseWriter, r *http.Request) {
		store.deleteTask(w, r)
	})

	return mux
}

func (s *taskStore) listTasks(w http.ResponseWriter) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	ids := make([]uint64, 0, len(s.tasks))
	for id := range s.tasks {
		ids = append(ids, id)
	}
	sort.Slice(ids, func(i, j int) bool { return ids[i] < ids[j] })

	out := make([]Task, 0, len(ids))
	for _, id := range ids {
		out = append(out, s.tasks[id])
	}
	writeJSON(w, http.StatusOK, out)
}

func (s *taskStore) createTask(w http.ResponseWriter, r *http.Request) {
	var req struct {
		Title string `json:"title"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "invalid JSON body")
		return
	}
	if strings.TrimSpace(req.Title) == "" {
		writeError(w, http.StatusBadRequest, "title is required")
		return
	}

	s.mu.Lock()
	id := s.next
	s.next++
	task := Task{ID: id, Title: req.Title, Done: false}
	s.tasks[id] = task
	s.mu.Unlock()

	writeJSON(w, http.StatusCreated, task)
}

func (s *taskStore) getTask(w http.ResponseWriter, r *http.Request) {
	id, ok := idFromRequest(r)
	if !ok {
		writeError(w, http.StatusBadRequest, "invalid task id")
		return
	}

	s.mu.RLock()
	task, exists := s.tasks[id]
	s.mu.RUnlock()

	if !exists {
		writeError(w, http.StatusNotFound, "task not found")
		return
	}
	writeJSON(w, http.StatusOK, task)
}

func (s *taskStore) updateTask(w http.ResponseWriter, r *http.Request) {
	id, ok := idFromRequest(r)
	if !ok {
		writeError(w, http.StatusBadRequest, "invalid task id")
		return
	}

	var req struct {
		Title *string `json:"title"`
		Done  *bool   `json:"done"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "invalid JSON body")
		return
	}
	if req.Title == nil && req.Done == nil {
		writeError(w, http.StatusBadRequest, "at least one field is required")
		return
	}
	if req.Title != nil && strings.TrimSpace(*req.Title) == "" {
		writeError(w, http.StatusBadRequest, "title cannot be empty")
		return
	}

	s.mu.Lock()
	task, exists := s.tasks[id]
	if !exists {
		s.mu.Unlock()
		writeError(w, http.StatusNotFound, "task not found")
		return
	}
	if req.Title != nil {
		task.Title = *req.Title
	}
	if req.Done != nil {
		task.Done = *req.Done
	}
	s.tasks[id] = task
	s.mu.Unlock()

	writeJSON(w, http.StatusOK, task)
}

func (s *taskStore) deleteTask(w http.ResponseWriter, r *http.Request) {
	id, ok := idFromRequest(r)
	if !ok {
		writeError(w, http.StatusBadRequest, "invalid task id")
		return
	}

	s.mu.Lock()
	_, exists := s.tasks[id]
	if exists {
		delete(s.tasks, id)
	}
	s.mu.Unlock()

	if !exists {
		writeError(w, http.StatusNotFound, "task not found")
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func idFromRequest(r *http.Request) (uint64, bool) {
	id, err := strconv.ParseUint(r.PathValue("id"), 10, 64)
	if err != nil || id == 0 {
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
	if err := http.ListenAndServe(":3000", App()); err != nil {
		panic(err)
	}
}

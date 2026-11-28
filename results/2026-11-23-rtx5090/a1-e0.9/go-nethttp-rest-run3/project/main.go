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
	mu    sync.RWMutex
	tasks map[uint64]Task
	nextID uint64
}

func newStore() *store {
	return &store{
		tasks: make(map[uint64]Task),
		nextID: 1,
	}
}

func writeJSON(w http.ResponseWriter, status int, v any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(v)
}

func writeError(w http.ResponseWriter, status int, msg string) {
	writeJSON(w, status, map[string]string{"error": msg})
}

func parseTaskID(r *http.Request) (uint64, error) {
	return strconv.ParseUint(r.PathValue("id"), 10, 64)
}

func App() http.Handler {
	s := newStore()

	mux := http.NewServeMux()

	mux.HandleFunc("GET /health", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, http.StatusOK, map[string]string{"status": "ok"})
	})

	mux.HandleFunc("GET /tasks", func(w http.ResponseWriter, r *http.Request) {
		s.mu.RLock()
		tasks := make([]Task, 0, len(s.tasks))
		for _, t := range s.tasks {
			tasks = append(tasks, t)
		}
		s.mu.RUnlock()

		sort.Slice(tasks, func(i, j int) bool { return tasks[i].ID < tasks[j].ID })
		writeJSON(w, http.StatusOK, tasks)
	})

	mux.HandleFunc("POST /tasks", func(w http.ResponseWriter, r *http.Request) {
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
		task := Task{
			ID:    s.nextID,
			Title: req.Title,
			Done:  false,
		}
		s.tasks[task.ID] = task
		s.nextID++
		s.mu.Unlock()

		writeJSON(w, http.StatusCreated, task)
	})

	mux.HandleFunc("GET /tasks/{id}", func(w http.ResponseWriter, r *http.Request) {
		id, err := parseTaskID(r)
		if err != nil {
			writeError(w, http.StatusBadRequest, "invalid task id")
			return
		}

		s.mu.RLock()
		task, ok := s.tasks[id]
		s.mu.RUnlock()
		if !ok {
			writeError(w, http.StatusNotFound, "task not found")
			return
		}

		writeJSON(w, http.StatusOK, task)
	})

	mux.HandleFunc("PUT /tasks/{id}", func(w http.ResponseWriter, r *http.Request) {
		id, err := parseTaskID(r)
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
		if strings.TrimSpace(req.Title) == "" {
			writeError(w, http.StatusBadRequest, "title is required")
			return
		}

		s.mu.Lock()
		task, ok := s.tasks[id]
		if !ok {
			s.mu.Unlock()
			writeError(w, http.StatusNotFound, "task not found")
			return
		}
		task.Title = req.Title
		task.Done = req.Done
		s.tasks[id] = task
		s.mu.Unlock()

		writeJSON(w, http.StatusOK, task)
	})

	mux.HandleFunc("DELETE /tasks/{id}", func(w http.ResponseWriter, r *http.Request) {
		id, err := parseTaskID(r)
		if err != nil {
			writeError(w, http.StatusBadRequest, "invalid task id")
			return
		}

		s.mu.Lock()
		_, ok := s.tasks[id]
		if !ok {
			s.mu.Unlock()
			writeError(w, http.StatusNotFound, "task not found")
			return
		}
		delete(s.tasks, id)
		s.mu.Unlock()

		w.WriteHeader(http.StatusNoContent)
	})

	return mux
}

func main() {
	log.Fatal(http.ListenAndServe(":3000", App()))
}

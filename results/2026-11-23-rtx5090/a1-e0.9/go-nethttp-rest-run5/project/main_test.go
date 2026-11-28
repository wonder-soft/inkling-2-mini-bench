package main

import (
	"encoding/json"
	"io"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
)

func do(t *testing.T, h http.Handler, method, path, body string) *httptest.ResponseRecorder {
	t.Helper()
	var rd io.Reader
	if body != "" {
		rd = strings.NewReader(body)
	}
	req := httptest.NewRequest(method, path, rd)
	if body != "" {
		req.Header.Set("Content-Type", "application/json")
	}
	rec := httptest.NewRecorder()
	h.ServeHTTP(rec, req)
	return rec
}

func TestHealth(t *testing.T) {
	h := App()
	rec := do(t, h, http.MethodGet, "/health", "")
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	var body map[string]string
	if err := json.NewDecoder(rec.Body).Decode(&body); err != nil {
		t.Fatalf("decode response: %v", err)
	}
	if body["status"] != "ok" {
		t.Fatalf("expected status ok, got %q", body["status"])
	}
}

func TestCreateTask(t *testing.T) {
	h := App()
	rec := do(t, h, http.MethodPost, "/tasks", `{"title":"Buy milk"}`)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d (%s)", rec.Code, rec.Body.String())
	}
	var task Task
	if err := json.NewDecoder(rec.Body).Decode(&task); err != nil {
		t.Fatalf("decode response: %v", err)
	}
	if task.ID != 1 {
		t.Fatalf("expected id 1, got %d", task.ID)
	}
	if task.Title != "Buy milk" {
		t.Fatalf("expected title %q, got %q", "Buy milk", task.Title)
	}
	if task.Done {
		t.Fatalf("expected done=false")
	}
}

func TestGetTaskAfterCreate(t *testing.T) {
	h := App()
	do(t, h, http.MethodPost, "/tasks", `{"title":"Write tests"}`)
	rec := do(t, h, http.MethodGet, "/tasks/1", "")
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	var task Task
	if err := json.NewDecoder(rec.Body).Decode(&task); err != nil {
		t.Fatalf("decode response: %v", err)
	}
	if task.ID != 1 || task.Title != "Write tests" || task.Done {
		t.Fatalf("unexpected task: %+v", task)
	}
}

func TestGetMissingTask(t *testing.T) {
	h := App()
	rec := do(t, h, http.MethodGet, "/tasks/999", "")
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", rec.Code)
	}
}

func TestDeleteTask(t *testing.T) {
	h := App()
	do(t, h, http.MethodPost, "/tasks", `{"title":"Delete me"}`)
	rec := do(t, h, http.MethodDelete, "/tasks/1", "")
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d", rec.Code)
	}
	rec = do(t, h, http.MethodGet, "/tasks/1", "")
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404 after delete, got %d", rec.Code)
	}
}

func TestUpdateTask(t *testing.T) {
	h := App()
	do(t, h, http.MethodPost, "/tasks", `{"title":"Old title"}`)
	rec := do(t, h, http.MethodPut, "/tasks/1", `{"title":"New title","done":true}`)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d (%s)", rec.Code, rec.Body.String())
	}
	var task Task
	if err := json.NewDecoder(rec.Body).Decode(&task); err != nil {
		t.Fatalf("decode response: %v", err)
	}
	if task.Title != "New title" || !task.Done {
		t.Fatalf("unexpected task: %+v", task)
	}
}

func TestListTasks(t *testing.T) {
	h := App()
	do(t, h, http.MethodPost, "/tasks", `{"title":"First"}`)
	do(t, h, http.MethodPost, "/tasks", `{"title":"Second"}`)
	rec := do(t, h, http.MethodGet, "/tasks", "")
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	var tasks []Task
	if err := json.NewDecoder(rec.Body).Decode(&tasks); err != nil {
		t.Fatalf("decode response: %v", err)
	}
	if len(tasks) != 2 || tasks[0].ID != 1 || tasks[1].ID != 2 {
		t.Fatalf("unexpected tasks: %+v", tasks)
	}
}

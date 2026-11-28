package main

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
)

func doRequest(t *testing.T, h http.Handler, method, path, body string) *httptest.ResponseRecorder {
	t.Helper()
	var req *http.Request
	if body != "" {
		req = httptest.NewRequest(method, path, strings.NewReader(body))
	} else {
		req = httptest.NewRequest(method, path, nil)
	}
	req.Header.Set("Content-Type", "application/json")
	rec := httptest.NewRecorder()
	h.ServeHTTP(rec, req)
	return rec
}

func TestHealth(t *testing.T) {
	h := App()
	rec := doRequest(t, h, "GET", "/health", "")
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
}

func TestCreateTask(t *testing.T) {
	h := App()
	rec := doRequest(t, h, "POST", "/tasks", `{"title":"Buy milk"}`)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d", rec.Code)
	}
	var task Task
	if err := json.NewDecoder(rec.Body).Decode(&task); err != nil {
		t.Fatalf("failed to decode response: %v", err)
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

func TestGetCreatedTask(t *testing.T) {
	h := App()
	doRequest(t, h, "POST", "/tasks", `{"title":"Buy milk"}`)
	rec := doRequest(t, h, "GET", "/tasks/1", "")
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	var task Task
	if err := json.NewDecoder(rec.Body).Decode(&task); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}
	if task.ID != 1 || task.Title != "Buy milk" || task.Done {
		t.Fatalf("unexpected task: %+v", task)
	}
}

func TestGetMissingTask(t *testing.T) {
	h := App()
	rec := doRequest(t, h, "GET", "/tasks/999", "")
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", rec.Code)
	}
}

func TestDeleteTask(t *testing.T) {
	h := App()
	doRequest(t, h, "POST", "/tasks", `{"title":"Buy milk"}`)
	rec := doRequest(t, h, "DELETE", "/tasks/1", "")
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d", rec.Code)
	}
	rec = doRequest(t, h, "GET", "/tasks/1", "")
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404 after delete, got %d", rec.Code)
	}
}

func TestUpdateTask(t *testing.T) {
	h := App()
	doRequest(t, h, "POST", "/tasks", `{"title":"Buy milk"}`)
	rec := doRequest(t, h, "PUT", "/tasks/1", `{"title":"Buy eggs","done":true}`)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	var task Task
	if err := json.NewDecoder(rec.Body).Decode(&task); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}
	if task.Title != "Buy eggs" || !task.Done {
		t.Fatalf("unexpected task: %+v", task)
	}
}

func TestListTasks(t *testing.T) {
	h := App()
	doRequest(t, h, "POST", "/tasks", `{"title":"First"}`)
	doRequest(t, h, "POST", "/tasks", `{"title":"Second"}`)
	rec := doRequest(t, h, "GET", "/tasks", "")
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	var tasks []Task
	if err := json.NewDecoder(rec.Body).Decode(&tasks); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}
	if len(tasks) != 2 || tasks[0].ID != 1 || tasks[1].ID != 2 {
		t.Fatalf("unexpected tasks: %+v", tasks)
	}
}

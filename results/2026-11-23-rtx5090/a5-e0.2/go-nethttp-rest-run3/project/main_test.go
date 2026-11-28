package main

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
)

func doRequest(t *testing.T, app http.Handler, method, path, body string) *httptest.ResponseRecorder {
	t.Helper()
	var reader *strings.Reader
	if body != "" {
		reader = strings.NewReader(body)
	} else {
		reader = strings.NewReader("")
	}
	req := httptest.NewRequest(method, path, reader)
	if body != "" {
		req.Header.Set("Content-Type", "application/json")
	}
	rec := httptest.NewRecorder()
	app.ServeHTTP(rec, req)
	return rec
}

func TestHealth(t *testing.T) {
	rec := doRequest(t, App(), http.MethodGet, "/health", "")
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
}

func TestCreateTask(t *testing.T) {
	rec := doRequest(t, App(), http.MethodPost, "/tasks", `{"title":"Buy milk"}`)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d", rec.Code)
	}
	var task Task
	if err := json.NewDecoder(rec.Body).Decode(&task); err != nil {
		t.Fatalf("decode response: %v", err)
	}
	if task.ID != 1 {
		t.Fatalf("expected id 1, got %d", task.ID)
	}
}

func TestGetCreatedTask(t *testing.T) {
	app := App()
	create := doRequest(t, app, http.MethodPost, "/tasks", `{"title":"Write tests"}`)
	if create.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d", create.Code)
	}
	rec := doRequest(t, app, http.MethodGet, "/tasks/1", "")
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
	rec := doRequest(t, App(), http.MethodGet, "/tasks/999", "")
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", rec.Code)
	}
}

func TestDeleteTask(t *testing.T) {
	app := App()
	create := doRequest(t, app, http.MethodPost, "/tasks", `{"title":"Delete me"}`)
	if create.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d", create.Code)
	}
	del := doRequest(t, app, http.MethodDelete, "/tasks/1", "")
	if del.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d", del.Code)
	}
	rec := doRequest(t, app, http.MethodGet, "/tasks/1", "")
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404 after delete, got %d", rec.Code)
	}
}

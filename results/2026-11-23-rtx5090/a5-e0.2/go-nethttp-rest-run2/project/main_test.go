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
	var req *http.Request
	if body != "" {
		req = httptest.NewRequest(method, path, strings.NewReader(body))
	} else {
		req = httptest.NewRequest(method, path, nil)
	}
	req.Header.Set("Content-Type", "application/json")
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
	app := App()
	rec := doRequest(t, app, http.MethodPost, "/tasks", `{"title":"Buy milk"}`)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d", rec.Code)
	}
	var task Task
	if err := json.NewDecoder(rec.Body).Decode(&task); err != nil {
		t.Fatalf("decode: %v", err)
	}
	if task.ID != 1 {
		t.Fatalf("expected id 1, got %d", task.ID)
	}
	if task.Title != "Buy milk" || task.Done {
		t.Fatalf("unexpected task: %+v", task)
	}
}

func TestGetTaskAfterCreate(t *testing.T) {
	app := App()
	doRequest(t, app, http.MethodPost, "/tasks", `{"title":"Write tests"}`)
	rec := doRequest(t, app, http.MethodGet, "/tasks/1", "")
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	var task Task
	if err := json.NewDecoder(rec.Body).Decode(&task); err != nil {
		t.Fatalf("decode: %v", err)
	}
	if task.ID != 1 || task.Title != "Write tests" {
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
	doRequest(t, app, http.MethodPost, "/tasks", `{"title":"Delete me"}`)
	rec := doRequest(t, app, http.MethodDelete, "/tasks/1", "")
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d", rec.Code)
	}
	rec = doRequest(t, app, http.MethodGet, "/tasks/1", "")
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404 after delete, got %d", rec.Code)
	}
}

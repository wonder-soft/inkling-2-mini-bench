package main

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

func doRequest(t *testing.T, app http.Handler, method, path, body string) *httptest.ResponseRecorder {
	t.Helper()

	req := httptest.NewRequest(method, path, bytes.NewReader([]byte(body)))
	rec := httptest.NewRecorder()
	app.ServeHTTP(rec, req)
	return rec
}

func createTask(t *testing.T, app http.Handler, title string) Task {
	t.Helper()

	rec := doRequest(t, app, http.MethodPost, "/tasks", `{"title":"`+title+`"}`)
	if rec.Code != http.StatusCreated {
		t.Fatalf("POST /tasks: expected status 201, got %d", rec.Code)
	}
	var task Task
	if err := json.NewDecoder(rec.Body).Decode(&task); err != nil {
		t.Fatalf("POST /tasks: failed to decode response: %v", err)
	}
	return task
}

func TestHealth(t *testing.T) {
	app := App()
	rec := doRequest(t, app, http.MethodGet, "/health", "")
	if rec.Code != http.StatusOK {
		t.Fatalf("GET /health: expected status 200, got %d", rec.Code)
	}
	var body map[string]string
	if err := json.NewDecoder(rec.Body).Decode(&body); err != nil {
		t.Fatalf("GET /health: failed to decode response: %v", err)
	}
	if body["status"] != "ok" {
		t.Fatalf("GET /health: expected status ok, got %q", body["status"])
	}
}

func TestCreateTask(t *testing.T) {
	app := App()
	rec := doRequest(t, app, http.MethodPost, "/tasks", `{"title":"Buy milk"}`)
	if rec.Code != http.StatusCreated {
		t.Fatalf("POST /tasks: expected status 201, got %d", rec.Code)
	}
	var task Task
	if err := json.NewDecoder(rec.Body).Decode(&task); err != nil {
		t.Fatalf("POST /tasks: failed to decode response: %v", err)
	}
	if task.ID != 1 {
		t.Fatalf("POST /tasks: expected id 1, got %d", task.ID)
	}
	if task.Title != "Buy milk" {
		t.Fatalf("POST /tasks: expected title %q, got %q", "Buy milk", task.Title)
	}
	if task.Done {
		t.Fatalf("POST /tasks: expected done=false")
	}
}

func TestGetTaskAfterCreate(t *testing.T) {
	app := App()
	created := createTask(t, app, "Write tests")

	rec := doRequest(t, app, http.MethodGet, "/tasks/1", "")
	if rec.Code != http.StatusOK {
		t.Fatalf("GET /tasks/1: expected status 200, got %d", rec.Code)
	}
	var task Task
	if err := json.NewDecoder(rec.Body).Decode(&task); err != nil {
		t.Fatalf("GET /tasks/1: failed to decode response: %v", err)
	}
	if task.ID != created.ID || task.Title != created.Title || task.Done != created.Done {
		t.Fatalf("GET /tasks/1: expected %+v, got %+v", created, task)
	}
}

func TestGetMissingTask(t *testing.T) {
	app := App()
	rec := doRequest(t, app, http.MethodGet, "/tasks/999", "")
	if rec.Code != http.StatusNotFound {
		t.Fatalf("GET /tasks/999: expected status 404, got %d", rec.Code)
	}
}

func TestDeleteTask(t *testing.T) {
	app := App()
	createTask(t, app, "Delete me")

	rec := doRequest(t, app, http.MethodDelete, "/tasks/1", "")
	if rec.Code != http.StatusNoContent {
		t.Fatalf("DELETE /tasks/1: expected status 204, got %d", rec.Code)
	}

	rec = doRequest(t, app, http.MethodGet, "/tasks/1", "")
	if rec.Code != http.StatusNotFound {
		t.Fatalf("GET /tasks/1 after delete: expected status 404, got %d", rec.Code)
	}
}

func TestUpdateTask(t *testing.T) {
	app := App()
	createTask(t, app, "Old title")

	rec := doRequest(t, app, http.MethodPut, "/tasks/1", `{"title":"New title","done":true}`)
	if rec.Code != http.StatusOK {
		t.Fatalf("PUT /tasks/1: expected status 200, got %d", rec.Code)
	}
	var task Task
	if err := json.NewDecoder(rec.Body).Decode(&task); err != nil {
		t.Fatalf("PUT /tasks/1: failed to decode response: %v", err)
	}
	if task.Title != "New title" || !task.Done {
		t.Fatalf("PUT /tasks/1: expected updated task, got %+v", task)
	}
}

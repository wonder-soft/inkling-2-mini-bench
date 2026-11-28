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

	var reader *strings.Reader
	if body == "" {
		reader = strings.NewReader("")
	} else {
		reader = strings.NewReader(body)
	}

	req := httptest.NewRequest(method, path, reader)
	if body != "" {
		req.Header.Set("Content-Type", "application/json")
	}

	rec := httptest.NewRecorder()
	h.ServeHTTP(rec, req)
	return rec
}

func TestHealth(t *testing.T) {
	rec := doRequest(t, App(), "GET", "/health", "")
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}

	var body map[string]string
	if err := json.NewDecoder(rec.Body).Decode(&body); err != nil {
		t.Fatalf("decode body: %v", err)
	}
	if body["status"] != "ok" {
		t.Fatalf("expected status ok, got %q", body["status"])
	}
}

func TestCreateTask(t *testing.T) {
	rec := doRequest(t, App(), "POST", "/tasks", `{"title":"first"}`)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d", rec.Code)
	}

	var task Task
	if err := json.NewDecoder(rec.Body).Decode(&task); err != nil {
		t.Fatalf("decode body: %v", err)
	}
	if task.ID != 1 {
		t.Fatalf("expected id 1, got %d", task.ID)
	}
	if task.Title != "first" {
		t.Fatalf("expected title first, got %q", task.Title)
	}
	if task.Done {
		t.Fatal("expected done=false")
	}
}

func TestGetCreatedTask(t *testing.T) {
	h := App()

	post := doRequest(t, h, "POST", "/tasks", `{"title":"second"}`)
	if post.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d", post.Code)
	}

	rec := doRequest(t, h, "GET", "/tasks/1", "")
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}

	var task Task
	if err := json.NewDecoder(rec.Body).Decode(&task); err != nil {
		t.Fatalf("decode body: %v", err)
	}
	if task.ID != 1 || task.Title != "second" || task.Done {
		t.Fatalf("unexpected task: %+v", task)
	}
}

func TestGetMissingTask(t *testing.T) {
	rec := doRequest(t, App(), "GET", "/tasks/999", "")
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", rec.Code)
	}
}

func TestDeleteTask(t *testing.T) {
	h := App()

	post := doRequest(t, h, "POST", "/tasks", `{"title":"third"}`)
	if post.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d", post.Code)
	}

	del := doRequest(t, h, "DELETE", "/tasks/1", "")
	if del.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d", del.Code)
	}

	rec := doRequest(t, h, "GET", "/tasks/1", "")
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", rec.Code)
	}
}

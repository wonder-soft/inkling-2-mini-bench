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

	req := httptest.NewRequest(method, path, strings.NewReader(body))
	if body != "" {
		req.Header.Set("Content-Type", "application/json")
	}

	rec := httptest.NewRecorder()
	h.ServeHTTP(rec, req)
	return rec
}

func decodeBody(t *testing.T, rec *httptest.ResponseRecorder, v any) {
	t.Helper()
	if err := json.NewDecoder(rec.Body).Decode(v); err != nil {
		t.Fatalf("failed to decode response body: %v", err)
	}
}

func TestHealth(t *testing.T) {
	rec := doRequest(t, App(), http.MethodGet, "/health", "")
	if rec.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d", rec.Code)
	}

	var body map[string]string
	decodeBody(t, rec, &body)
	if body["status"] != "ok" {
		t.Fatalf("expected status ok, got %q", body["status"])
	}
}

func TestCreateTask(t *testing.T) {
	rec := doRequest(t, App(), http.MethodPost, "/tasks", `{"title":"Buy milk"}`)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected status 201, got %d", rec.Code)
	}

	var task Task
	decodeBody(t, rec, &task)
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

	postRec := doRequest(t, h, http.MethodPost, "/tasks", `{"title":"Write tests"}`)
	if postRec.Code != http.StatusCreated {
		t.Fatalf("expected status 201, got %d", postRec.Code)
	}

	rec := doRequest(t, h, http.MethodGet, "/tasks/1", "")
	if rec.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d", rec.Code)
	}

	var task Task
	decodeBody(t, rec, &task)
	if task.ID != 1 {
		t.Fatalf("expected id 1, got %d", task.ID)
	}
	if task.Title != "Write tests" {
		t.Fatalf("expected title %q, got %q", "Write tests", task.Title)
	}
	if task.Done {
		t.Fatalf("expected done=false")
	}
}

func TestGetMissingTask(t *testing.T) {
	rec := doRequest(t, App(), http.MethodGet, "/tasks/999", "")
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected status 404, got %d", rec.Code)
	}
}

func TestDeleteTask(t *testing.T) {
	h := App()

	postRec := doRequest(t, h, http.MethodPost, "/tasks", `{"title":"Delete me"}`)
	if postRec.Code != http.StatusCreated {
		t.Fatalf("expected status 201, got %d", postRec.Code)
	}

	deleteRec := doRequest(t, h, http.MethodDelete, "/tasks/1", "")
	if deleteRec.Code != http.StatusNoContent {
		t.Fatalf("expected status 204, got %d", deleteRec.Code)
	}

	getRec := doRequest(t, h, http.MethodGet, "/tasks/1", "")
	if getRec.Code != http.StatusNotFound {
		t.Fatalf("expected status 404 after delete, got %d", getRec.Code)
	}
}

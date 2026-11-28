package main

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
)

func doRequest(t *testing.T, handler http.Handler, method, path, body string) *httptest.ResponseRecorder {
	t.Helper()
	var req *http.Request
	if body != "" {
		req = httptest.NewRequest(method, path, strings.NewReader(body))
	} else {
		req = httptest.NewRequest(method, path, nil)
	}
	req.Header.Set("Content-Type", "application/json")
	rec := httptest.NewRecorder()
	handler.ServeHTTP(rec, req)
	return rec
}

func TestHealth(t *testing.T) {
	rec := doRequest(t, App(), http.MethodGet, "/health", "")
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
}

func TestCreateAndGetTask(t *testing.T) {
	handler := App()

	rec := doRequest(t, handler, http.MethodPost, "/tasks", `{"title":"Buy milk"}`)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d", rec.Code)
	}
	var created Task
	if err := json.NewDecoder(rec.Body).Decode(&created); err != nil {
		t.Fatalf("decode created task: %v", err)
	}
	if created.ID != 1 {
		t.Fatalf("expected id 1, got %d", created.ID)
	}

	rec = doRequest(t, handler, http.MethodGet, "/tasks/1", "")
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	var fetched Task
	if err := json.NewDecoder(rec.Body).Decode(&fetched); err != nil {
		t.Fatalf("decode fetched task: %v", err)
	}
	if fetched.ID != 1 || fetched.Title != "Buy milk" || fetched.Done {
		t.Fatalf("unexpected task: %+v", fetched)
	}
}

func TestGetMissingTask(t *testing.T) {
	rec := doRequest(t, App(), http.MethodGet, "/tasks/999", "")
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", rec.Code)
	}
}

func TestDeleteTask(t *testing.T) {
	handler := App()
	doRequest(t, handler, http.MethodPost, "/tasks", `{"title":"Temp"}`)

	rec := doRequest(t, handler, http.MethodDelete, "/tasks/1", "")
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d", rec.Code)
	}

	rec = doRequest(t, handler, http.MethodGet, "/tasks/1", "")
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404 after delete, got %d", rec.Code)
	}
}

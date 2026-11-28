package main

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
)

func TestHealth(t *testing.T) {
	rec := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/health", nil)
	App().ServeHTTP(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
}

func TestCreateTask(t *testing.T) {
	h := App()
	rec := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodPost, "/tasks", strings.NewReader(`{"title":"Buy milk"}`))
	req.Header.Set("Content-Type", "application/json")
	h.ServeHTTP(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d", rec.Code)
	}
	var task Task
	if err := json.NewDecoder(rec.Body).Decode(&task); err != nil {
		t.Fatalf("decoding response: %v", err)
	}
	if task.ID != 1 {
		t.Fatalf("expected id 1, got %d", task.ID)
	}
	if task.Title != "Buy milk" {
		t.Fatalf("expected title 'Buy milk', got %q", task.Title)
	}
	if task.Done {
		t.Fatalf("expected done=false")
	}
}

func TestGetCreatedTask(t *testing.T) {
	h := App()
	postRec := httptest.NewRecorder()
	postReq := httptest.NewRequest(http.MethodPost, "/tasks", strings.NewReader(`{"title":"Write tests"}`))
	h.ServeHTTP(postRec, postReq)

	rec := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/tasks/1", nil)
	h.ServeHTTP(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	var task Task
	if err := json.NewDecoder(rec.Body).Decode(&task); err != nil {
		t.Fatalf("decoding response: %v", err)
	}
	if task.ID != 1 || task.Title != "Write tests" {
		t.Fatalf("unexpected task: %+v", task)
	}
}

func TestGetMissingTask(t *testing.T) {
	rec := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/tasks/999", nil)
	App().ServeHTTP(rec, req)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", rec.Code)
	}
}

func TestDeleteTask(t *testing.T) {
	h := App()
	postRec := httptest.NewRecorder()
	postReq := httptest.NewRequest(http.MethodPost, "/tasks", strings.NewReader(`{"title":"Delete me"}`))
	h.ServeHTTP(postRec, postReq)

	delRec := httptest.NewRecorder()
	delReq := httptest.NewRequest(http.MethodDelete, "/tasks/1", nil)
	h.ServeHTTP(delRec, delReq)
	if delRec.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d", delRec.Code)
	}

	getRec := httptest.NewRecorder()
	getReq := httptest.NewRequest(http.MethodGet, "/tasks/1", nil)
	h.ServeHTTP(getRec, getReq)
	if getRec.Code != http.StatusNotFound {
		t.Fatalf("expected 404 after delete, got %d", getRec.Code)
	}
}

func TestUpdateTask(t *testing.T) {
	h := App()
	postRec := httptest.NewRecorder()
	postReq := httptest.NewRequest(http.MethodPost, "/tasks", strings.NewReader(`{"title":"Old title"}`))
	h.ServeHTTP(postRec, postReq)

	putRec := httptest.NewRecorder()
	putReq := httptest.NewRequest(http.MethodPut, "/tasks/1", strings.NewReader(`{"title":"New title","done":true}`))
	h.ServeHTTP(putRec, putReq)
	if putRec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", putRec.Code)
	}
	var task Task
	if err := json.NewDecoder(putRec.Body).Decode(&task); err != nil {
		t.Fatalf("decoding response: %v", err)
	}
	if task.Title != "New title" || !task.Done {
		t.Fatalf("unexpected task: %+v", task)
	}
}

func TestListTasksOrderedByID(t *testing.T) {
	h := App()
	for _, title := range []string{"one", "two", "three"} {
		rec := httptest.NewRecorder()
		req := httptest.NewRequest(http.MethodPost, "/tasks", strings.NewReader(`{"title":"`+title+`"}`))
		h.ServeHTTP(rec, req)
	}
	rec := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/tasks", nil)
	h.ServeHTTP(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	var tasks []Task
	if err := json.NewDecoder(rec.Body).Decode(&tasks); err != nil {
		t.Fatalf("decoding response: %v", err)
	}
	if len(tasks) != 3 {
		t.Fatalf("expected 3 tasks, got %d", len(tasks))
	}
	for i, task := range tasks {
		if task.ID != uint64(i+1) {
			t.Fatalf("expected task %d with id %d, got %d", i, i+1, task.ID)
		}
	}
}

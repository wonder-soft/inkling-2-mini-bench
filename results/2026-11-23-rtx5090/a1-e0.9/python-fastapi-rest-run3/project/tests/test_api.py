import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from fastapi.testclient import TestClient

from app.main import create_app


def test_health_returns_200():
    client = TestClient(create_app())
    resp = client.get("/health")
    assert resp.status_code == 200
    assert resp.json() == {"status": "ok"}


def test_create_task_returns_201_and_id_1():
    client = TestClient(create_app())
    resp = client.post("/tasks", json={"title": "Buy milk"})
    assert resp.status_code == 201
    data = resp.json()
    assert data["id"] == 1
    assert data["title"] == "Buy milk"
    assert data["done"] is False


def test_get_task_after_creation():
    client = TestClient(create_app())
    client.post("/tasks", json={"title": "Task A"})
    resp = client.get("/tasks/1")
    assert resp.status_code == 200
    assert resp.json() == {"id": 1, "title": "Task A", "done": False}


def test_get_missing_task_returns_404():
    client = TestClient(create_app())
    resp = client.get("/tasks/999")
    assert resp.status_code == 404


def test_delete_existing_task_returns_204_then_404():
    client = TestClient(create_app())
    client.post("/tasks", json={"title": "To delete"})
    resp = client.delete("/tasks/1")
    assert resp.status_code == 204
    resp = client.get("/tasks/1")
    assert resp.status_code == 404


def test_list_tasks_ordered_by_id():
    client = TestClient(create_app())
    client.post("/tasks", json={"title": "First"})
    client.post("/tasks", json={"title": "Second"})
    resp = client.get("/tasks")
    assert resp.status_code == 200
    assert [t["id"] for t in resp.json()] == [1, 2]


def test_update_task_returns_updated_task():
    client = TestClient(create_app())
    client.post("/tasks", json={"title": "Old title"})
    resp = client.put("/tasks/1", json={"title": "New title", "done": True})
    assert resp.status_code == 200
    assert resp.json() == {"id": 1, "title": "New title", "done": True}


def test_update_missing_task_returns_404():
    client = TestClient(create_app())
    resp = client.put("/tasks/999", json={"title": "X", "done": True})
    assert resp.status_code == 404


def test_delete_missing_task_returns_404():
    client = TestClient(create_app())
    resp = client.delete("/tasks/999")
    assert resp.status_code == 404

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from fastapi.testclient import TestClient

from app.main import create_app


def make_client():
    return TestClient(create_app())


def test_health_returns_200():
    client = make_client()
    response = client.get("/health")
    assert response.status_code == 200
    assert response.json() == {"status": "ok"}


def test_create_task_returns_201_and_id_1():
    client = make_client()
    response = client.post("/tasks", json={"title": "Buy milk"})
    assert response.status_code == 201
    data = response.json()
    assert data["id"] == 1
    assert data["title"] == "Buy milk"
    assert data["done"] is False


def test_get_task_after_creation():
    client = make_client()
    client.post("/tasks", json={"title": "Write tests"})
    response = client.get("/tasks/1")
    assert response.status_code == 200
    assert response.json() == {"id": 1, "title": "Write tests", "done": False}


def test_get_missing_task_returns_404():
    client = make_client()
    response = client.get("/tasks/999")
    assert response.status_code == 404


def test_delete_task_returns_204_then_404():
    client = make_client()
    client.post("/tasks", json={"title": "Temporary"})
    response = client.delete("/tasks/1")
    assert response.status_code == 204
    response = client.get("/tasks/1")
    assert response.status_code == 404


def test_list_tasks_ordered_by_id():
    client = make_client()
    client.post("/tasks", json={"title": "First"})
    client.post("/tasks", json={"title": "Second"})
    response = client.get("/tasks")
    assert response.status_code == 200
    assert [t["id"] for t in response.json()] == [1, 2]


def test_update_task_returns_updated_task():
    client = make_client()
    client.post("/tasks", json={"title": "Old title"})
    response = client.put("/tasks/1", json={"title": "New title", "done": True})
    assert response.status_code == 200
    assert response.json() == {"id": 1, "title": "New title", "done": True}


def test_update_missing_task_returns_404():
    client = make_client()
    response = client.put("/tasks/999", json={"title": "Nope", "done": False})
    assert response.status_code == 404


def test_delete_missing_task_returns_404():
    client = make_client()
    response = client.delete("/tasks/999")
    assert response.status_code == 404

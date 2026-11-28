import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from fastapi.testclient import TestClient

from app.main import create_app


def test_health_returns_200():
    client = TestClient(create_app())
    response = client.get("/health")
    assert response.status_code == 200
    assert response.json() == {"status": "ok"}


def test_post_task_returns_201_and_id_1():
    client = TestClient(create_app())
    response = client.post("/tasks", json={"title": "Buy milk"})
    assert response.status_code == 201
    body = response.json()
    assert body["id"] == 1
    assert body["title"] == "Buy milk"
    assert body["done"] is False


def test_get_task_after_creation():
    client = TestClient(create_app())
    client.post("/tasks", json={"title": "Write tests"})
    response = client.get("/tasks/1")
    assert response.status_code == 200
    body = response.json()
    assert body["id"] == 1
    assert body["title"] == "Write tests"
    assert body["done"] is False


def test_get_missing_task_returns_404():
    client = TestClient(create_app())
    response = client.get("/tasks/999")
    assert response.status_code == 404


def test_delete_existing_task_returns_204_then_404():
    client = TestClient(create_app())
    client.post("/tasks", json={"title": "Delete me"})
    response = client.delete("/tasks/1")
    assert response.status_code == 204
    assert client.get("/tasks/1").status_code == 404


def test_list_tasks_ordered_by_id():
    client = TestClient(create_app())
    client.post("/tasks", json={"title": "first"})
    client.post("/tasks", json={"title": "second"})
    response = client.get("/tasks")
    assert response.status_code == 200
    assert [task["id"] for task in response.json()] == [1, 2]


def test_put_updates_existing_task():
    client = TestClient(create_app())
    client.post("/tasks", json={"title": "old title"})
    response = client.put("/tasks/1", json={"title": "new title", "done": True})
    assert response.status_code == 200
    body = response.json()
    assert body == {"id": 1, "title": "new title", "done": True}


def test_put_missing_task_returns_404():
    client = TestClient(create_app())
    response = client.put("/tasks/999", json={"title": "x", "done": True})
    assert response.status_code == 404


def test_delete_missing_task_returns_404():
    client = TestClient(create_app())
    assert client.delete("/tasks/999").status_code == 404

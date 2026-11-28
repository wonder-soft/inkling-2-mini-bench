from fastapi.testclient import TestClient

from app.main import create_app


def make_client():
    return TestClient(create_app())


def test_health():
    client = make_client()
    resp = client.get("/health")
    assert resp.status_code == 200
    assert resp.json() == {"status": "ok"}


def test_create_task():
    client = make_client()
    resp = client.post("/tasks", json={"title": "Buy milk"})
    assert resp.status_code == 201
    data = resp.json()
    assert data["id"] == 1
    assert data["title"] == "Buy milk"
    assert data["done"] is False


def test_get_task_after_creation():
    client = make_client()
    client.post("/tasks", json={"title": "Write tests"})
    resp = client.get("/tasks/1")
    assert resp.status_code == 200
    assert resp.json() == {"id": 1, "title": "Write tests", "done": False}


def test_get_missing_task_returns_404():
    client = make_client()
    resp = client.get("/tasks/999")
    assert resp.status_code == 404


def test_delete_task_returns_204_then_404():
    client = make_client()
    client.post("/tasks", json={"title": "Delete me"})
    resp = client.delete("/tasks/1")
    assert resp.status_code == 204
    resp = client.get("/tasks/1")
    assert resp.status_code == 404

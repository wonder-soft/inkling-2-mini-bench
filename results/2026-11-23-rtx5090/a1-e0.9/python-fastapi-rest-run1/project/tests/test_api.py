from fastapi.testclient import TestClient

from app.main import create_app


def make_client() -> TestClient:
    return TestClient(create_app())


def test_health():
    client = make_client()
    resp = client.get("/health")
    assert resp.status_code == 200
    assert resp.json() == {"status": "ok"}


def test_create_task_returns_201_and_id_1():
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
    data = resp.json()
    assert data["id"] == 1
    assert data["title"] == "Write tests"
    assert data["done"] is False


def test_get_missing_task_returns_404():
    client = make_client()
    resp = client.get("/tasks/999")
    assert resp.status_code == 404


def test_delete_task_returns_204_then_get_404():
    client = make_client()
    client.post("/tasks", json={"title": "Temporary"})
    resp = client.delete("/tasks/1")
    assert resp.status_code == 204
    resp = client.get("/tasks/1")
    assert resp.status_code == 404


def test_list_tasks_ordered_by_id():
    client = make_client()
    client.post("/tasks", json={"title": "First"})
    client.post("/tasks", json={"title": "Second"})
    resp = client.get("/tasks")
    assert resp.status_code == 200
    assert [t["id"] for t in resp.json()] == [1, 2]


def test_update_task():
    client = make_client()
    client.post("/tasks", json={"title": "Old title"})
    resp = client.put("/tasks/1", json={"title": "New title", "done": True})
    assert resp.status_code == 200
    data = resp.json()
    assert data["title"] == "New title"
    assert data["done"] is True


def test_update_missing_task_returns_404():
    client = make_client()
    resp = client.put("/tasks/999", json={"title": "Nope", "done": False})
    assert resp.status_code == 404

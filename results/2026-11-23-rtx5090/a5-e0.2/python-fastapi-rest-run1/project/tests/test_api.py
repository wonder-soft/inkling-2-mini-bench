from fastapi.testclient import TestClient

from app.main import create_app


def make_client():
    return TestClient(create_app())


def test_health():
    client = make_client()
    response = client.get("/health")
    assert response.status_code == 200


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
    data = response.json()
    assert data["id"] == 1
    assert data["title"] == "Write tests"
    assert data["done"] is False


def test_get_missing_task_returns_404():
    client = make_client()
    response = client.get("/tasks/999")
    assert response.status_code == 404


def test_delete_task_returns_204_then_404():
    client = make_client()
    client.post("/tasks", json={"title": "Delete me"})
    response = client.delete("/tasks/1")
    assert response.status_code == 204
    response = client.get("/tasks/1")
    assert response.status_code == 404

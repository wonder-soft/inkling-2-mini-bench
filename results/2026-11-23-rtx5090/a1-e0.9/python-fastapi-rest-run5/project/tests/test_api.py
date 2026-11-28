from fastapi.testclient import TestClient

from app.main import create_app


def test_health_returns_200():
    client = TestClient(create_app())
    response = client.get("/health")
    assert response.status_code == 200
    assert response.json() == {"status": "ok"}


def test_create_task_returns_201_and_id_1():
    client = TestClient(create_app())
    response = client.post("/tasks", json={"title": "Buy milk"})
    assert response.status_code == 201
    data = response.json()
    assert data["id"] == 1
    assert data["title"] == "Buy milk"
    assert data["done"] is False


def test_get_task_after_creation_returns_task():
    client = TestClient(create_app())
    created = client.post("/tasks", json={"title": "Write docs"}).json()
    response = client.get(f"/tasks/{created['id']}")
    assert response.status_code == 200
    assert response.json() == created


def test_get_missing_task_returns_404():
    client = TestClient(create_app())
    response = client.get("/tasks/999")
    assert response.status_code == 404


def test_delete_existing_task_returns_204_and_subsequent_get_404():
    client = TestClient(create_app())
    created = client.post("/tasks", json={"title": "Delete me"}).json()
    response = client.delete(f"/tasks/{created['id']}")
    assert response.status_code == 204
    response = client.get(f"/tasks/{created['id']}")
    assert response.status_code == 404


def test_update_existing_task_returns_updated_task():
    client = TestClient(create_app())
    created = client.post("/tasks", json={"title": "Old title"}).json()
    response = client.put(f"/tasks/{created['id']}", json={"title": "New title", "done": True})
    assert response.status_code == 200
    data = response.json()
    assert data == {"id": created["id"], "title": "New title", "done": True}


def test_list_tasks_returns_tasks_ordered_by_id():
    client = TestClient(create_app())
    first = client.post("/tasks", json={"title": "First"}).json()
    second = client.post("/tasks", json={"title": "Second"}).json()
    response = client.get("/tasks")
    assert response.status_code == 200
    assert response.json() == [first, second]

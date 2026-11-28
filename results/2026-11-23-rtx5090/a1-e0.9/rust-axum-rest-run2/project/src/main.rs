use axum::{
    body::Body,
    extract::{Path, State},
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Task {
    id: u64,
    title: String,
    done: bool,
}

#[derive(Debug, Default)]
struct StateInner {
    tasks: Vec<Task>,
    next_id: u64,
}

#[derive(Clone)]
struct AppState {
    inner: Arc<RwLock<StateInner>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            inner: Arc::new(RwLock::new(StateInner {
                tasks: Vec::new(),
                next_id: 1,
            })),
        }
    }
}

#[derive(Debug, Deserialize)]
struct CreateTaskRequest {
    title: String,
}

#[derive(Debug, Deserialize)]
struct UpdateTaskRequest {
    title: String,
    done: bool,
}

pub fn app() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/tasks", get(list_tasks).post(create_task))
        .route(
            "/tasks/{id}",
            get(get_task).put(update_task).delete(delete_task),
        )
        .with_state(AppState::default())
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn list_tasks(State(state): State<AppState>) -> Json<Vec<Task>> {
    let inner = state.inner.read().unwrap();
    let mut tasks = inner.tasks.clone();
    tasks.sort_by_key(|t| t.id);
    Json(tasks)
}

async fn create_task(
    State(state): State<AppState>,
    Json(req): Json<CreateTaskRequest>,
) -> (StatusCode, Json<Task>) {
    let mut inner = state.inner.write().unwrap();
    let task = Task {
        id: inner.next_id,
        title: req.title,
        done: false,
    };
    inner.next_id += 1;
    inner.tasks.push(task.clone());
    (StatusCode::CREATED, Json(task))
}

async fn get_task(State(state): State<AppState>, Path(id): Path<u64>) -> Response {
    let inner = state.inner.read().unwrap();
    match inner.tasks.iter().find(|t| t.id == id) {
        Some(task) => Json(task.clone()).into_response(),
        None => not_found(),
    }
}

async fn update_task(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(req): Json<UpdateTaskRequest>,
) -> Response {
    let mut inner = state.inner.write().unwrap();
    match inner.tasks.iter_mut().find(|t| t.id == id) {
        Some(task) => {
            task.title = req.title;
            task.done = req.done;
            Json(task.clone()).into_response()
        }
        None => not_found(),
    }
}

async fn delete_task(State(state): State<AppState>, Path(id): Path<u64>) -> Response {
    let mut inner = state.inner.write().unwrap();
    if let Some(pos) = inner.tasks.iter().position(|t| t.id == id) {
        inner.tasks.remove(pos);
        StatusCode::NO_CONTENT.into_response()
    } else {
        not_found()
    }
}

fn not_found() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "task not found" })),
    )
        .into_response()
}

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app()).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn response_json(response: Response) -> serde_json::Value {
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    fn json_request(method: &str, uri: &str, body: Option<&str>) -> Request<Body> {
        let mut builder = Request::builder().method(method).uri(uri);
        if body.is_some() {
            builder = builder.header("content-type", "application/json");
        }
        builder
            .body(match body {
                Some(b) => Body::from(b.to_string()),
                None => Body::empty(),
            })
            .unwrap()
    }

    #[tokio::test]
    async fn health_returns_200() {
        let app = app();
        let response = app
            .oneshot(json_request("GET", "/health", None))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        assert_eq!(body["status"], "ok");
    }

    #[tokio::test]
    async fn create_task_returns_201_and_id_1() {
        let app = app();
        let response = app
            .oneshot(json_request("POST", "/tasks", Some(r#"{"title":"Test task"}"#)))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let body = response_json(response).await;
        assert_eq!(body["id"], 1);
        assert_eq!(body["title"], "Test task");
        assert_eq!(body["done"], false);
    }

    #[tokio::test]
    async fn get_task_after_creation_returns_task() {
        let app = app();
        let response = app
            .clone()
            .oneshot(json_request("POST", "/tasks", Some(r#"{"title":"Test task"}"#)))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let response = app
            .oneshot(json_request("GET", "/tasks/1", None))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        assert_eq!(body["id"], 1);
        assert_eq!(body["title"], "Test task");
        assert_eq!(body["done"], false);
    }

    #[tokio::test]
    async fn get_missing_task_returns_404() {
        let app = app();
        let response = app
            .oneshot(json_request("GET", "/tasks/999", None))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delete_existing_task_returns_204_then_404() {
        let app = app();
        let response = app
            .clone()
            .oneshot(json_request("POST", "/tasks", Some(r#"{"title":"Test task"}"#)))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let response = app
            .clone()
            .oneshot(json_request("DELETE", "/tasks/1", None))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        let response = app
            .oneshot(json_request("GET", "/tasks/1", None))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}

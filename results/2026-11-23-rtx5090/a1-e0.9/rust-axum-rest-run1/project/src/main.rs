use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Task {
    id: u64,
    title: String,
    done: bool,
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

#[derive(Default)]
struct AppState {
    tasks: HashMap<u64, Task>,
    next_id: u64,
}

type SharedState = Arc<RwLock<AppState>>;

pub fn app() -> Router {
    let state = Arc::new(RwLock::new(AppState {
        tasks: HashMap::new(),
        next_id: 1,
    }));

    Router::new()
        .route("/health", get(health))
        .route("/tasks", get(list_tasks).post(create_task))
        .route(
            "/tasks/{id}",
            get(get_task).put(update_task).delete(delete_task),
        )
        .with_state(state)
}

async fn health() -> impl IntoResponse {
    Json(json!({ "status": "ok" }))
}

async fn list_tasks(State(state): State<SharedState>) -> impl IntoResponse {
    let state = state.read().unwrap();
    let mut tasks: Vec<Task> = state.tasks.values().cloned().collect();
    tasks.sort_by_key(|task| task.id);
    Json(tasks)
}

async fn create_task(
    State(state): State<SharedState>,
    Json(payload): Json<CreateTaskRequest>,
) -> impl IntoResponse {
    let mut state = state.write().unwrap();
    let id = state.next_id;
    state.next_id += 1;
    let task = Task {
        id,
        title: payload.title,
        done: false,
    };
    state.tasks.insert(id, task.clone());
    (StatusCode::CREATED, Json(task))
}

async fn get_task(State(state): State<SharedState>, Path(id): Path<u64>) -> Response {
    let state = state.read().unwrap();
    match state.tasks.get(&id) {
        Some(task) => Json(task.clone()).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "task not found" })),
        )
            .into_response(),
    }
}

async fn update_task(
    State(state): State<SharedState>,
    Path(id): Path<u64>,
    Json(payload): Json<UpdateTaskRequest>,
) -> Response {
    let mut state = state.write().unwrap();
    match state.tasks.get_mut(&id) {
        Some(task) => {
            task.title = payload.title;
            task.done = payload.done;
            Json(task.clone()).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "task not found" })),
        )
            .into_response(),
    }
}

async fn delete_task(State(state): State<SharedState>, Path(id): Path<u64>) -> Response {
    let mut state = state.write().unwrap();
    if state.tasks.remove(&id).is_some() {
        StatusCode::NO_CONTENT.into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "task not found" })),
        )
            .into_response()
    }
}

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app()).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn body_json(response: Response) -> Value {
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    fn json_request(method: &str, uri: &str, body: Option<Value>) -> Request<Body> {
        let mut builder = Request::builder().method(method).uri(uri);
        let body = match body {
            Some(value) => {
                builder = builder.header("content-type", "application/json");
                Body::from(serde_json::to_vec(&value).unwrap())
            }
            None => Body::empty(),
        };
        builder.body(body).unwrap()
    }

    #[tokio::test]
    async fn health_returns_200() {
        let app = app();
        let response = app
            .oneshot(json_request("GET", "/health", None))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = body_json(response).await;
        assert_eq!(body["status"], "ok");
    }

    #[tokio::test]
    async fn create_task_returns_201_with_id_1() {
        let app = app();
        let response = app
            .oneshot(json_request(
                "POST",
                "/tasks",
                Some(json!({ "title": "Buy milk" })),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let body = body_json(response).await;
        assert_eq!(body["id"], 1);
        assert_eq!(body["title"], "Buy milk");
        assert_eq!(body["done"], false);
    }

    #[tokio::test]
    async fn get_created_task_returns_task() {
        let app = app();
        let response = app
            .clone()
            .oneshot(json_request(
                "POST",
                "/tasks",
                Some(json!({ "title": "Write tests" })),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let response = app
            .oneshot(json_request("GET", "/tasks/1", None))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = body_json(response).await;
        assert_eq!(body["id"], 1);
        assert_eq!(body["title"], "Write tests");
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
    async fn delete_task_returns_204_then_get_returns_404() {
        let app = app();
        let response = app
            .clone()
            .oneshot(json_request(
                "POST",
                "/tasks",
                Some(json!({ "title": "Temp" })),
            ))
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

    #[tokio::test]
    async fn update_task_returns_updated_task() {
        let app = app();
        let response = app
            .clone()
            .oneshot(json_request(
                "POST",
                "/tasks",
                Some(json!({ "title": "Old" })),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let response = app
            .clone()
            .oneshot(json_request(
                "PUT",
                "/tasks/1",
                Some(json!({ "title": "New", "done": true })),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = body_json(response).await;
        assert_eq!(body["title"], "New");
        assert_eq!(body["done"], true);
    }
}

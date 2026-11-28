use axum::{
    body::Body,
    extract::{Path, State},
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post, put},
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

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: &'static str,
}

#[derive(Deserialize)]
struct CreateTaskRequest {
    title: String,
}

#[derive(Deserialize)]
struct UpdateTaskRequest {
    title: String,
    done: bool,
}

#[derive(Default)]
struct AppState {
    tasks: Vec<Task>,
    next_id: u64,
}

impl AppState {
    fn new() -> Self {
        Self {
            tasks: Vec::new(),
            next_id: 1,
        }
    }
}

type SharedState = Arc<RwLock<AppState>>;

pub fn app() -> Router {
    let state: SharedState = Arc::new(RwLock::new(AppState::new()));
    Router::new()
        .route("/health", get(health))
        .route("/tasks", get(list_tasks).post(create_task))
        .route(
            "/tasks/{id}",
            get(get_task).put(update_task).delete(delete_task),
        )
        .with_state(state)
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn list_tasks(State(state): State<SharedState>) -> Json<Vec<Task>> {
    let state = state.read().unwrap();
    let mut tasks = state.tasks.clone();
    tasks.sort_by_key(|t| t.id);
    Json(tasks)
}

async fn create_task(
    State(state): State<SharedState>,
    Json(payload): Json<CreateTaskRequest>,
) -> (StatusCode, Json<Task>) {
    let mut state = state.write().unwrap();
    let task = Task {
        id: state.next_id,
        title: payload.title,
        done: false,
    };
    state.next_id += 1;
    state.tasks.push(task.clone());
    (StatusCode::CREATED, Json(task))
}

async fn get_task(State(state): State<SharedState>, Path(id): Path<u64>) -> Response {
    let state = state.read().unwrap();
    match state.tasks.iter().find(|t| t.id == id) {
        Some(task) => Json(task.clone()).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "task not found",
            }),
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
    match state.tasks.iter_mut().find(|t| t.id == id) {
        Some(task) => {
            task.title = payload.title;
            task.done = payload.done;
            Json(task.clone()).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "task not found",
            }),
        )
            .into_response(),
    }
}

async fn delete_task(State(state): State<SharedState>, Path(id): Path<u64>) -> Response {
    let mut state = state.write().unwrap();
    match state.tasks.iter().position(|t| t.id == id) {
        Some(pos) => {
            state.tasks.remove(pos);
            StatusCode::NO_CONTENT.into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "task not found",
            }),
        )
            .into_response(),
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
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn body_json<T: serde::de::DeserializeOwned>(resp: Response) -> T {
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
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
        let resp = app()
            .oneshot(json_request("GET", "/health", None))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn create_task_returns_201_and_id_1() {
        let resp = app()
            .oneshot(json_request("POST", "/tasks", Some(r#"{"title":"Buy milk"}"#)))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let task: Task = body_json(resp).await;
        assert_eq!(task.id, 1);
        assert_eq!(task.title, "Buy milk");
        assert!(!task.done);
    }

    #[tokio::test]
    async fn get_task_after_creation_returns_task() {
        let app = app();
        let resp = app
            .clone()
            .oneshot(json_request("POST", "/tasks", Some(r#"{"title":"Write tests"}"#)))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let resp = app
            .oneshot(json_request("GET", "/tasks/1", None))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let task: Task = body_json(resp).await;
        assert_eq!(task.id, 1);
        assert_eq!(task.title, "Write tests");
    }

    #[tokio::test]
    async fn get_missing_task_returns_404() {
        let resp = app()
            .oneshot(json_request("GET", "/tasks/999", None))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delete_existing_task_returns_204_then_404() {
        let app = app();
        let resp = app
            .clone()
            .oneshot(json_request("POST", "/tasks", Some(r#"{"title":"Delete me"}"#)))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let resp = app
            .clone()
            .oneshot(json_request("DELETE", "/tasks/1", None))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

        let resp = app
            .oneshot(json_request("GET", "/tasks/1", None))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}

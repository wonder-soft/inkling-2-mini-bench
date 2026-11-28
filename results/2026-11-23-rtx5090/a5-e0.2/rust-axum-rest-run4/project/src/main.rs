use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct Task {
    id: u64,
    title: String,
    done: bool,
}

#[derive(Deserialize)]
struct CreateTask {
    title: String,
}

#[derive(Deserialize)]
struct UpdateTask {
    title: String,
    done: bool,
}

#[derive(Default)]
struct StateInner {
    tasks: Vec<Task>,
    next_id: u64,
}

impl StateInner {
    fn new() -> Self {
        StateInner {
            tasks: Vec::new(),
            next_id: 1,
        }
    }
}

type SharedState = Arc<RwLock<StateInner>>;

fn not_found() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "not found" })),
    )
        .into_response()
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn list_tasks(State(state): State<SharedState>) -> Json<Vec<Task>> {
    let state = state.read().unwrap();
    let mut tasks = state.tasks.clone();
    tasks.sort_by_key(|t| t.id);
    Json(tasks)
}

async fn create_task(
    State(state): State<SharedState>,
    Json(body): Json<CreateTask>,
) -> (StatusCode, Json<Task>) {
    let mut state = state.write().unwrap();
    let task = Task {
        id: state.next_id,
        title: body.title,
        done: false,
    };
    state.next_id += 1;
    state.tasks.push(task.clone());
    (StatusCode::CREATED, Json(task))
}

async fn get_task(
    State(state): State<SharedState>,
    Path(id): Path<u64>,
) -> Response {
    let state = state.read().unwrap();
    match state.tasks.iter().find(|t| t.id == id) {
        Some(task) => Json(task.clone()).into_response(),
        None => not_found(),
    }
}

async fn update_task(
    State(state): State<SharedState>,
    Path(id): Path<u64>,
    Json(body): Json<UpdateTask>,
) -> Response {
    let mut state = state.write().unwrap();
    match state.tasks.iter_mut().find(|t| t.id == id) {
        Some(task) => {
            task.title = body.title;
            task.done = body.done;
            Json(task.clone()).into_response()
        }
        None => not_found(),
    }
}

async fn delete_task(
    State(state): State<SharedState>,
    Path(id): Path<u64>,
) -> Response {
    let mut state = state.write().unwrap();
    let before = state.tasks.len();
    state.tasks.retain(|t| t.id != id);
    if state.tasks.len() < before {
        StatusCode::NO_CONTENT.into_response()
    } else {
        not_found()
    }
}

pub fn app() -> Router {
    let state: SharedState = Arc::new(RwLock::new(StateInner::new()));
    Router::new()
        .route("/health", get(health))
        .route("/tasks", get(list_tasks).post(create_task))
        .route(
            "/tasks/{id}",
            get(get_task).put(update_task).delete(delete_task),
        )
        .with_state(state)
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
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use serde::de::DeserializeOwned;
    use tower::ServiceExt;

    async fn json_body<T: DeserializeOwned>(response: Response) -> T {
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn health_returns_200() {
        let response = app()
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn create_task_returns_201_and_id_1() {
        let response = app()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/tasks")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"title":"buy milk"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let task: Task = json_body(response).await;
        assert_eq!(task.id, 1);
        assert_eq!(task.title, "buy milk");
        assert!(!task.done);
    }

    #[tokio::test]
    async fn get_task_after_creation_returns_task() {
        let router = app();
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/tasks")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"title":"write tests"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let response = router
            .oneshot(Request::builder().uri("/tasks/1").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let task: Task = json_body(response).await;
        assert_eq!(task.id, 1);
        assert_eq!(task.title, "write tests");
    }

    #[tokio::test]
    async fn get_missing_task_returns_404() {
        let response = app()
            .oneshot(Request::builder().uri("/tasks/999").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delete_task_returns_204_then_404() {
        let router = app();
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/tasks")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"title":"delete me"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let response = router
            .clone()
            .oneshot(Request::builder().method("DELETE").uri("/tasks/1").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        let response = router
            .oneshot(Request::builder().uri("/tasks/1").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}

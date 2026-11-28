use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct Task {
    id: u64,
    title: String,
    done: bool,
}

#[derive(Debug, Deserialize)]
struct CreateTask {
    title: String,
}

#[derive(Debug, Deserialize)]
struct UpdateTask {
    title: String,
    done: bool,
}

#[derive(Default)]
struct AppState {
    tasks: Vec<Task>,
    next_id: u64,
}

type SharedState = Arc<RwLock<AppState>>;

pub fn app() -> Router {
    let state: SharedState = Arc::new(RwLock::new(AppState::default()));
    Router::new()
        .route("/health", get(health))
        .route("/tasks", get(list_tasks).post(create_task))
        .route(
            "/tasks/{id}",
            get(get_task).put(update_task).delete(delete_task),
        )
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn list_tasks(State(state): State<SharedState>) -> Json<Vec<Task>> {
    let state = state.read().unwrap();
    Json(state.tasks.clone())
}

async fn create_task(
    State(state): State<SharedState>,
    Json(payload): Json<CreateTask>,
) -> (StatusCode, Json<Task>) {
    let mut state = state.write().unwrap();
    state.next_id += 1;
    let task = Task {
        id: state.next_id,
        title: payload.title,
        done: false,
    };
    state.tasks.push(task.clone());
    (StatusCode::CREATED, Json(task))
}

async fn get_task(
    State(state): State<SharedState>,
    Path(id): Path<u64>,
) -> Result<Json<Task>, StatusCode> {
    let state = state.read().unwrap();
    state
        .tasks
        .iter()
        .find(|t| t.id == id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn update_task(
    State(state): State<SharedState>,
    Path(id): Path<u64>,
    Json(payload): Json<UpdateTask>,
) -> Result<Json<Task>, StatusCode> {
    let mut state = state.write().unwrap();
    let task = state.tasks.iter_mut().find(|t| t.id == id).ok_or(StatusCode::NOT_FOUND)?;
    task.title = payload.title;
    task.done = payload.done;
    Ok(Json(task.clone()))
}

async fn delete_task(
    State(state): State<SharedState>,
    Path(id): Path<u64>,
) -> StatusCode {
    let mut state = state.write().unwrap();
    let before = state.tasks.len();
    state.tasks.retain(|t| t.id != id);
    if state.tasks.len() < before {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
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
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn body_json<T: serde::de::DeserializeOwned>(
        response: axum::response::Response,
    ) -> T {
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
                    .body(Body::from(r#"{"title":"write tests"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let task: Task = body_json(response).await;
        assert_eq!(task.id, 1);
        assert_eq!(task.title, "write tests");
        assert!(!task.done);
    }

    #[tokio::test]
    async fn get_task_after_creation() {
        let app = app();
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/tasks")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"title":"fetch me"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let response = app
            .oneshot(Request::builder().uri("/tasks/1").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let task: Task = body_json(response).await;
        assert_eq!(task.id, 1);
        assert_eq!(task.title, "fetch me");
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
        let app = app();
        let response = app
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

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/tasks/1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        let response = app
            .oneshot(Request::builder().uri("/tasks/1").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}

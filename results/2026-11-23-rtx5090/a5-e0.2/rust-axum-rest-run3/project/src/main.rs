use axum::{
    extract::{Path, State},
    http::StatusCode,
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
struct AppState {
    tasks: Vec<Task>,
    next_id: u64,
}

impl AppState {
    fn new() -> Self {
        AppState {
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
    Json(payload): Json<CreateTask>,
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

    async fn body_json(resp: axum::response::Response) -> serde_json::Value {
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn health_returns_200() {
        let resp = app()
            .oneshot(Request::get("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn create_task_returns_201_and_id_1() {
        let resp = app()
            .oneshot(
                Request::post("/tasks")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"title":"Buy milk"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let body = body_json(resp).await;
        assert_eq!(body["id"], 1);
        assert_eq!(body["title"], "Buy milk");
        assert_eq!(body["done"], false);
    }

    #[tokio::test]
    async fn get_task_after_creation_returns_task() {
        let app = app();
        let resp = app
            .clone()
            .oneshot(
                Request::post("/tasks")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"title":"Write tests"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let resp = app
            .oneshot(Request::get("/tasks/1").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["id"], 1);
        assert_eq!(body["title"], "Write tests");
        assert_eq!(body["done"], false);
    }

    #[tokio::test]
    async fn get_missing_task_returns_404() {
        let resp = app()
            .oneshot(Request::get("/tasks/999").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delete_task_returns_204_then_get_404() {
        let app = app();
        let resp = app
            .clone()
            .oneshot(
                Request::post("/tasks")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"title":"Delete me"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let resp = app
            .clone()
            .oneshot(Request::delete("/tasks/1").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

        let resp = app
            .oneshot(Request::get("/tasks/1").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}

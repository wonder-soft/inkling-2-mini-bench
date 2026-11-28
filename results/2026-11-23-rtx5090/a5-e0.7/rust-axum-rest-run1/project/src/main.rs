use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
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
struct CreateTaskRequest {
    title: String,
}

#[derive(Deserialize)]
struct UpdateTaskRequest {
    title: String,
    done: bool,
}

struct AppState {
    tasks: Vec<Task>,
    next_id: u64,
}

type SharedState = Arc<RwLock<AppState>>;

pub fn app() -> Router {
    let state: SharedState = Arc::new(RwLock::new(AppState {
        tasks: Vec::new(),
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

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn list_tasks(State(state): State<SharedState>) -> Json<Vec<Task>> {
    let state = state.read().unwrap();
    Json(state.tasks.clone())
}

async fn create_task(
    State(state): State<SharedState>,
    Json(req): Json<CreateTaskRequest>,
) -> (StatusCode, Json<Task>) {
    let mut state = state.write().unwrap();
    let task = Task {
        id: state.next_id,
        title: req.title,
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
            Json(serde_json::json!({ "error": "task not found" })),
        )
            .into_response(),
    }
}

async fn update_task(
    State(state): State<SharedState>,
    Path(id): Path<u64>,
    Json(req): Json<UpdateTaskRequest>,
) -> Response {
    let mut state = state.write().unwrap();
    match state.tasks.iter_mut().find(|t| t.id == id) {
        Some(task) => {
            task.title = req.title;
            task.done = req.done;
            Json(task.clone()).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "task not found" })),
        )
            .into_response(),
    }
}

async fn delete_task(State(state): State<SharedState>, Path(id): Path<u64>) -> Response {
    let mut state = state.write().unwrap();
    if let Some(pos) = state.tasks.iter().position(|t| t.id == id) {
        state.tasks.remove(pos);
        StatusCode::NO_CONTENT.into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "task not found" })),
        )
            .into_response()
    }
}

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind to 0.0.0.0:3000");
    axum::serve(listener, app()).await.expect("server error");
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn body_json(resp: Response) -> serde_json::Value {
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
                    .body(Body::from(r#"{"title":"write tests"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let v = body_json(resp).await;
        assert_eq!(v["id"], 1);
        assert_eq!(v["title"], "write tests");
        assert_eq!(v["done"], false);
    }

    #[tokio::test]
    async fn get_created_task_returns_it() {
        let app = app();
        let resp = app
            .clone()
            .oneshot(
                Request::post("/tasks")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"title":"fetch me"}"#))
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
        let v = body_json(resp).await;
        assert_eq!(v["id"], 1);
        assert_eq!(v["title"], "fetch me");
        assert_eq!(v["done"], false);
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
    async fn delete_existing_task_returns_204_then_404() {
        let app = app();
        let resp = app
            .clone()
            .oneshot(
                Request::post("/tasks")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"title":"delete me"}"#))
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

use std::sync::{Arc, RwLock};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Task {
    id: u64,
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

#[derive(Deserialize)]
struct CreateTask {
    title: String,
}

#[derive(Deserialize)]
struct UpdateTask {
    title: String,
    done: bool,
}

type SharedState = Arc<RwLock<AppState>>;

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
    if let Some(task) = state.tasks.iter_mut().find(|t| t.id == id) {
        task.title = payload.title;
        task.done = payload.done;
        Ok(Json(task.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn delete_task(
    State(state): State<SharedState>,
    Path(id): Path<u64>,
) -> StatusCode {
    let mut state = state.write().unwrap();
    if let Some(pos) = state.tasks.iter().position(|t| t.id == id) {
        state.tasks.remove(pos);
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

pub fn app() -> Router {
    let state = Arc::new(RwLock::new(AppState::new()));
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
    use super::app;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::response::Response;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn body_bytes(resp: Response) -> Vec<u8> {
        resp.into_body().collect().await.unwrap().to_bytes().to_vec()
    }

    fn get_request(uri: &str) -> Request<Body> {
        Request::builder().uri(uri).body(Body::empty()).unwrap()
    }

    fn post_task_request(title: &str) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri("/tasks")
            .header("content-type", "application/json")
            .body(Body::from(format!(r#"{{"title":"{}"}}"#, title)))
            .unwrap()
    }

    #[tokio::test]
    async fn health_returns_200() {
        let resp = app().oneshot(get_request("/health")).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json: serde_json::Value =
            serde_json::from_slice(&body_bytes(resp).await).unwrap();
        assert_eq!(json["status"], "ok");
    }

    #[tokio::test]
    async fn create_task_returns_201_and_id_1() {
        let resp = app().oneshot(post_task_request("Buy milk")).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let json: serde_json::Value =
            serde_json::from_slice(&body_bytes(resp).await).unwrap();
        assert_eq!(json["id"], 1);
        assert_eq!(json["title"], "Buy milk");
        assert_eq!(json["done"], false);
    }

    #[tokio::test]
    async fn get_created_task_returns_200() {
        let app = app();
        let resp = app.clone().oneshot(post_task_request("Buy milk")).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let resp = app.oneshot(get_request("/tasks/1")).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json: serde_json::Value =
            serde_json::from_slice(&body_bytes(resp).await).unwrap();
        assert_eq!(json["id"], 1);
        assert_eq!(json["title"], "Buy milk");
        assert_eq!(json["done"], false);
    }

    #[tokio::test]
    async fn get_missing_task_returns_404() {
        let resp = app().oneshot(get_request("/tasks/999")).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delete_task_returns_204_then_get_returns_404() {
        let app = app();
        let resp = app.clone().oneshot(post_task_request("Buy milk")).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let resp = app
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
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

        let resp = app.oneshot(get_request("/tasks/1")).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn update_task_returns_200_with_changes() {
        let app = app();
        let resp = app.clone().oneshot(post_task_request("Buy milk")).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/tasks/1")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"title":"Buy eggs","done":true}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json: serde_json::Value =
            serde_json::from_slice(&body_bytes(resp).await).unwrap();
        assert_eq!(json["id"], 1);
        assert_eq!(json["title"], "Buy eggs");
        assert_eq!(json["done"], true);
    }
}

use std::sync::{Arc, RwLock};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize)]
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

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

struct Store {
    tasks: Vec<Task>,
    next_id: u64,
}

type AppState = Arc<RwLock<Store>>;

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn list_tasks(State(state): State<AppState>) -> Json<Vec<Task>> {
    let state = state.lock().unwrap();
    Json(state.tasks.clone())
}

async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTask>,
) -> (StatusCode, Json<Task>) {
    let mut state = state.lock().unwrap();
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
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<Task>, StatusCode> {
    let state = state.lock().unwrap();
    state
        .tasks
        .iter()
        .find(|t| t.id == id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn update_task(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(payload): Json<UpdateTask>,
) -> Result<Json<Task>, StatusCode> {
    let mut state = state.lock().unwrap();
    let task = state
        .tasks
        .iter_mut()
        .find(|t| t.id == id)
        .ok_or(StatusCode::NOT_FOUND)?;
    task.title = payload.title;
    task.done = payload.done;
    Ok(Json(task.clone()))
}

async fn delete_task(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> StatusCode {
    let mut state = state.lock().unwrap();
    let before = state.tasks.len();
    state.tasks.retain(|t| t.id != id);
    if state.tasks.len() < before {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

pub fn app() -> axum::Router {
    let state: AppState = Arc::new(RwLock::new(Store {
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

    async fn json_body(res: axum::response::Response) -> serde_json::Value {
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    fn post_request(body: &str) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri("/tasks")
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    fn get_request(uri: &str) -> Request<Body> {
        Request::builder().uri(uri).body(Body::empty()).unwrap()
    }

    #[tokio::test]
    async fn health_returns_200() {
        let res = app().oneshot(get_request("/health")).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let v = json_body(res).await;
        assert_eq!(v["status"], "ok");
    }

    #[tokio::test]
    async fn post_creates_task_with_id_1() {
        let res = app()
            .oneshot(post_request(r#"{"title":"Test task"}"#))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);
        let v = json_body(res).await;
        assert_eq!(v["id"], 1);
        assert_eq!(v["title"], "Test task");
        assert_eq!(v["done"], false);
    }

    #[tokio::test]
    async fn get_created_task_returns_task() {
        let app = app();
        let res = app.clone().oneshot(post_request(r#"{"title":"Test task"}"#)).await.unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);

        let res = app.clone().oneshot(get_request("/tasks/1")).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let v = json_body(res).await;
        assert_eq!(v["id"], 1);
        assert_eq!(v["title"], "Test task");
        assert_eq!(v["done"], false);
    }

    #[tokio::test]
    async fn get_missing_task_returns_404() {
        let res = app().oneshot(get_request("/tasks/999")).await.unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delete_existing_task_returns_204_then_404() {
        let app = app();
        let res = app.clone().oneshot(post_request(r#"{"title":"Test task"}"#)).await.unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);

        let res = app
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
        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let res = app.clone().oneshot(get_request("/tasks/1")).await.unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }
}

use std::sync::{Arc, RwLock};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct Task {
    id: u64,
    title: String,
    done: bool,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
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

struct Inner {
    tasks: Vec<Task>,
    next_id: u64,
}

type Shared = Arc<RwLock<Inner>>;

pub fn app() -> Router {
    let state: Shared = Arc::new(RwLock::new(Inner {
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

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn list_tasks(State(state): State<Shared>) -> Json<Vec<Task>> {
    let inner = state.read().unwrap();
    Json(inner.tasks.clone())
}

async fn create_task(
    State(state): State<Shared>,
    Json(payload): Json<CreateTask>,
) -> (StatusCode, Json<Task>) {
    let mut inner = state.write().unwrap();
    let task = Task {
        id: inner.next_id,
        title: payload.title,
        done: false,
    };
    inner.next_id += 1;
    inner.tasks.push(task.clone());
    (StatusCode::CREATED, Json(task))
}

async fn get_task(
    State(state): State<Shared>,
    Path(id): Path<u64>,
) -> Result<Json<Task>, StatusCode> {
    let inner = state.read().unwrap();
    inner
        .tasks
        .iter()
        .find(|t| t.id == id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn update_task(
    State(state): State<Shared>,
    Path(id): Path<u64>,
    Json(payload): Json<UpdateTask>,
) -> Result<Json<Task>, StatusCode> {
    let mut inner = state.write().unwrap();
    if let Some(task) = inner.tasks.iter_mut().find(|t| t.id == id) {
        task.title = payload.title;
        task.done = payload.done;
        Ok(Json(task.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn delete_task(
    State(state): State<Shared>,
    Path(id): Path<u64>,
) -> Result<StatusCode, StatusCode> {
    let mut inner = state.write().unwrap();
    let before = inner.tasks.len();
    inner.tasks.retain(|t| t.id != id);
    if inner.tasks.len() < before {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
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
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        response::Response,
    };
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
        builder.body(Body::from(body.unwrap_or(""))).unwrap()
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
    async fn post_task_returns_201_and_id_1() {
        let resp = app()
            .oneshot(json_request(
                "POST",
                "/tasks",
                Some(r#"{"title":"buy milk"}"#),
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let task: Task = body_json(resp).await;
        assert_eq!(task.id, 1);
        assert_eq!(task.title, "buy milk");
        assert!(!task.done);
    }

    #[tokio::test]
    async fn get_created_task_returns_it() {
        let app = app();
        let resp = app
            .clone()
            .oneshot(json_request(
                "POST",
                "/tasks",
                Some(r#"{"title":"write docs"}"#),
            ))
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
        assert_eq!(task.title, "write docs");
        assert!(!task.done);
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
    async fn delete_task_returns_204_then_get_returns_404() {
        let app = app();
        let resp = app
            .clone()
            .oneshot(json_request(
                "POST",
                "/tasks",
                Some(r#"{"title":"delete me"}"#),
            ))
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

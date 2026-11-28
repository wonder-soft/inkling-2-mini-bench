use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

struct Inner {
    tasks: Vec<Task>,
    next_id: u64,
}

type Shared = Arc<RwLock<Inner>>;

fn new_state() -> Shared {
    Arc::new(RwLock::new(Inner {
        tasks: Vec::new(),
        next_id: 1,
    }))
}

pub fn app() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/tasks", get(list_tasks).post(create_task))
        .route(
            "/tasks/{id}",
            get(get_task).put(update_task).delete(delete_task),
        )
        .with_state(new_state())
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

async fn list_tasks(State(state): State<Shared>) -> Json<Vec<Task>> {
    let inner = state.read().unwrap();
    Json(inner.tasks.iter().cloned().collect())
}

async fn create_task(
    State(state): State<Shared>,
    Json(payload): Json<CreateTask>,
) -> (StatusCode, Json<Task>) {
    let mut inner = state.write().unwrap();
    let id = inner.next_id;
    inner.next_id += 1;
    let task = Task {
        id,
        title: payload.title,
        done: false,
    };
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
) -> StatusCode {
    let mut inner = state.write().unwrap();
    let before = inner.tasks.len();
    inner.tasks.retain(|t| t.id != id);
    if inner.tasks.len() < before {
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
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn json_request(
        method: &str,
        uri: &str,
        body: Option<serde_json::Value>,
    ) -> axum::extract::Request {
        let body = match body {
            Some(v) => axum::body::Body::from(v.to_string()),
            None => axum::body::Body::empty(),
        };
        axum::http::Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json")
            .body(body)
            .unwrap()
    }

    async fn json_body(resp: axum::response::Response) -> serde_json::Value {
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
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
            .oneshot(json_request(
                "POST",
                "/tasks",
                Some(serde_json::json!({"title": "Buy milk"})),
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let v = json_body(resp).await;
        assert_eq!(v["id"], 1);
        assert_eq!(v["title"], "Buy milk");
        assert_eq!(v["done"], false);
    }

    #[tokio::test]
    async fn get_created_task_returns_200() {
        let app = app();
        let resp = app
            .clone()
            .oneshot(json_request(
                "POST",
                "/tasks",
                Some(serde_json::json!({"title": "Write tests"})),
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let resp = app
            .oneshot(json_request("GET", "/tasks/1", None))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let v = json_body(resp).await;
        assert_eq!(v["id"], 1);
        assert_eq!(v["title"], "Write tests");
        assert_eq!(v["done"], false);
    }

    #[tokio::test]
    async fn missing_task_returns_404() {
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
            .oneshot(json_request(
                "POST",
                "/tasks",
                Some(serde_json::json!({"title": "Temp"})),
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

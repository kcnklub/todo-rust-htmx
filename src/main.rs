use askama::Template;
use axum::extract::Form;
use axum::extract::State;
use axum::response::Html;
use axum::routing::get;
use axum::routing::put;
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct Task {
    id: String,
    title: String,
    completed: bool,
}

#[derive(Debug, serde::Deserialize)]
struct TaskRequest {
    title: String,
}

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

    let data: Arc<Mutex<HashMap<String, Task>>> = Arc::new(Mutex::new(HashMap::new()));

    // Serve static assets
    let serve_assets = tower_http::services::ServeDir::new("assets");

    let app = axum::Router::new()
        .route("/", get(home))
        .route("/tasks", get(get_tasks).post(create_task))
        .route("/tasks/:id", put(complete_task))
        .nest_service("/assets", serve_assets)
        .with_state(data);

    axum::serve(listener, app).await.unwrap();
}

pub async fn home() -> Html<String> {
    let content = fs::read_to_string("./templates/index.html").unwrap();
    Html(content)
}

type TaskState = Arc<Mutex<HashMap<String, Task>>>;

pub async fn get_tasks(State(tasks): State<TaskState>) -> Html<String> {
    let mut content = String::new();

    for task in tasks.lock().unwrap().clone().into_iter() {
        println!("{:?}", task);
        let template = TaskTemplate {
            id: task.1.id,
            title: task.1.title,
            completed: task.1.completed,
        };
        content.push_str(&template.render().unwrap());
    }

    Html(content)
}

#[derive(Template)]
#[template(path = "task.html")]
struct TaskTemplate {
    id: String,
    title: String,
    completed: bool,
}

async fn create_task(
    State(state): State<TaskState>,
    Form(request): Form<TaskRequest>,
) -> Html<String> {
    let id = uuid::Uuid::new_v4().to_string();
    state.lock().unwrap().insert(
        id.clone(),
        Task {
            id: id.clone(),
            title: request.title.clone(),
            completed: false,
        },
    );
    let template = TaskTemplate {
        id: id.clone(),
        title: request.title.clone(),
        completed: false,
    };
    let res = template.render().unwrap();
    Html(res)
}

async fn complete_task(
    State(state): State<TaskState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Html<String> {
    if let Some(task) = state.lock().unwrap().get_mut(&id) {
        let updated = !task.completed;
        task.completed = updated;
        let template = TaskTemplate {
            id: task.id.clone(),
            title: task.title.clone(),
            completed: task.completed,
        };
        let res = template.render().unwrap();
        Html(res)
    } else {
        Html("Task not found".to_string())
    }
}

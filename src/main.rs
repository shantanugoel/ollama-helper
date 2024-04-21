use anyhow::Result;
use axum::{
    body::Body,
    extract::{Request, State},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use hyper::{StatusCode, Uri};
use hyper_util::{client::legacy::connect::HttpConnector, rt::TokioExecutor};

type Client = hyper_util::client::legacy::Client<HttpConnector, Body>;

// Create a static mutex to hold state of ollama app
static OLLAMA_RUNNING: std::sync::Mutex<bool> = std::sync::Mutex::new(false);

#[tokio::main]
async fn main() {
    let client: Client =
        hyper_util::client::legacy::Client::<(), ()>::builder(TokioExecutor::new())
            .build(HttpConnector::new());
    let app = Router::new()
        .route("/*path", post(proxy).get(proxy))
        .with_state(client);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

enum OllamaRunnerCmd {
    Start,
    Stop,
}

fn ollama_runner(cmd: OllamaRunnerCmd) -> Result<String> {
    match cmd {
        OllamaRunnerCmd::Start => {
            // TODO: Add code to actually run ollama
            *OLLAMA_RUNNING.lock().unwrap() = true;
            Ok("Ollama started".to_string())
        }
        OllamaRunnerCmd::Stop => {
            // TODO: Add code to stop ollama
            *OLLAMA_RUNNING.lock().unwrap() = false;
            Ok("Ollama stopped".to_string())
        }
    }
}

async fn proxy(State(client): State<Client>, mut req: Request) -> Result<Response, StatusCode> {
    let path = req.uri().path();
    let path_query = req
        .uri()
        .path_and_query()
        .map(|v| v.as_str())
        .unwrap_or(path);

    let uri = format!("http://localhost:11434{}", path_query);
    println!("{}", uri);

    *req.uri_mut() = Uri::try_from(uri).unwrap();
    Ok(client
        .request(req)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .into_response())
}

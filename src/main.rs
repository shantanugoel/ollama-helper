use std::process::Stdio;

use anyhow::{bail, Result};
use axum::{
    body::Body,
    extract::{Request, State},
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use hyper::{StatusCode, Uri};
use hyper_util::{client::legacy::connect::HttpConnector, rt::TokioExecutor};
use tokio::process::{Child, Command};

type Client = hyper_util::client::legacy::Client<HttpConnector, Body>;

// Create a static mutex to hold state of ollama app
static OLLAMA_RUNNING: std::sync::Mutex<Option<Child>> = std::sync::Mutex::new(None);

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

async fn ollama_runner() -> Result<String> {
    if OLLAMA_RUNNING.lock().unwrap().is_none() {
        // TODO: Make the process name/path configurable
        let output = Command::new("C:\\Users\\shant\\AppData\\Local\\Programs\\Ollama\\ollama.exe")
            .arg("-v")
            .output()
            .await
            .unwrap();
        if output.status.success() {
            if std::str::from_utf8(output.stdout.as_slice())
                .unwrap()
                .contains("could not connect")
            {
                // TODO: Ideally we should be able to just use serve here and run models as needed but this somehow fails
                // Find and fix this. A potential workaround in the worst case would be to extract the model name from
                // the incoming request and run that model before proxying that request
                let child =
                    Command::new("C:\\Users\\shant\\AppData\\Local\\Programs\\Ollama\\ollama.exe")
                        .arg("run")
                        .arg("llama3")
                        .stdin(Stdio::null())
                        .spawn()?;
                // TODO: Add an idle timeout to shutdown ollama
                *OLLAMA_RUNNING.lock().unwrap() = Some(child);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        } else {
            bail!("Could not start ollama")
        }
    }
    Ok("Started".to_string())
}

async fn proxy(State(client): State<Client>, mut req: Request) -> Result<Response, StatusCode> {
    if ollama_runner().await.is_ok() {
        let path = req.uri().path();
        let path_query = req
            .uri()
            .path_and_query()
            .map(|v| v.as_str())
            .unwrap_or(path);

        // TODO: Make the url/port configurable
        let uri = format!("http://localhost:11434{}", path_query);
        println!("{}", uri);

        *req.uri_mut() = Uri::try_from(uri).unwrap();
        Ok(client
            .request(req)
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?
            .into_response())
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

use serde::{Deserialize, Serialize};
use tauri::State;

fn api_base() -> String {
    std::env::var("CONDENSR_API_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShortenResponse {
    pub code: String,
    pub short_url: String,
    pub long_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinkItem {
    pub id: u64,
    pub long_url: String,
    pub created_at: String,
    pub code: String,
}

#[derive(Serialize)]
struct ShortenRequest<'a> {
    url: &'a str,
}

#[derive(Deserialize)]
struct ApiError {
    description: String,
}

async fn error_message(resp: reqwest::Response) -> String {
    let status = resp.status();
    match resp.json::<ApiError>().await {
        Ok(body) => format!("{status}: {}", body.description),
        Err(_) => format!("request failed with status {status}"),
    }
}

#[tauri::command]
async fn shorten(
    client: State<'_, reqwest::Client>,
    url: String,
) -> Result<ShortenResponse, String> {
    let resp = client
        .post(format!("{}/api/shorten", api_base()))
        .json(&ShortenRequest { url: &url })
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(error_message(resp).await);
    }
    resp.json::<ShortenResponse>()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_links(
    client: State<'_, reqwest::Client>,
) -> Result<Vec<LinkItem>, String> {
    let resp = client
        .get(format!("{}/api/links", api_base()))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(error_message(resp).await);
    }
    resp.json::<Vec<LinkItem>>()
        .await
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(reqwest::Client::new())
        .invoke_handler(tauri::generate_handler![shorten, list_links])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

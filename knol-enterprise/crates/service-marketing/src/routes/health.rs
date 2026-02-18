use axum::Json;

pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "marketing",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

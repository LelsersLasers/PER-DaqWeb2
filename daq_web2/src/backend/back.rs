use dioxus::prelude::*;

use crate::config;

#[cfg(feature = "server")]
use crate::backend;

#[server]
pub async fn create_test(input: String) -> Result<String, ServerFnError> {
    tracing::info!("in create_test with input: {}", input);
    let db = backend::db::get_db_pool().await;
    Ok(format!("okay test with input: {}", input))
}

use dioxus::prelude::*;

#[cfg(feature = "server")]
use crate::s_helpers;

#[server]
pub async fn create_test(input: String) -> Result<String, ServerFnError> {
    let db = s_helpers::db::get_db_pool().await;
    Ok(format!("Created test with input: {}", input))
}

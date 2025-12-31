
use crate::s_helpers;

pub enum LogLevel {
    Info,
    Warning,
    Error,
}

impl LogLevel {
    // Must match the CHECK clause in the database schema
    pub fn to_string(&self) -> &str {
        match self {
            LogLevel::Info => "info",
            LogLevel::Warning => "warning",
            LogLevel::Error => "error",
        }
    }
}

pub async fn insert_log(upload_id: usize, level: LogLevel, message: &str) {
    let db = s_helpers::db::get_db_pool().await;
    let timestamp_server = chrono::Utc::now().naive_utc();
    let query = "INSERT INTO ProcessingInfos (upload_id, timestamp_server, info_type, string) VALUES (?, ?, ?, ?)";
    let _ = sqlx::query(query)
        .bind(upload_id as i64)
        .bind(timestamp_server.format("%Y-%m-%d %H:%M:%S").to_string())
        .bind(level.to_string())
        .bind(message)
        .execute(db)
        .await;
}
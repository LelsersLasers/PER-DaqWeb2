use crate::config;

pub async fn connect_db() -> sqlx::SqlitePool {
	let db_options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(config::SQLITE_DB)
        .create_if_missing(false);
   	sqlx::SqlitePool::connect_with(db_options)
        .await
        .expect("Failed to connect to SQLite database")
}
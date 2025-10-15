use crate::config;

static G_APP_STATE: tokio::sync::OnceCell<sqlx::SqlitePool> = tokio::sync::OnceCell::const_new();

async fn connect_db() -> sqlx::SqlitePool {
    let db_options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(config::SQLITE_DB)
        .create_if_missing(false);
    sqlx::SqlitePool::connect_with(db_options)
        .await
        .expect("Failed to connect to SQLite database")
}

pub async fn get_db_pool() -> &'static sqlx::SqlitePool {
    G_APP_STATE.get_or_init(connect_db).await
}

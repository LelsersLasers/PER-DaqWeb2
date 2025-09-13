#[derive(Clone)]
pub struct AppState {
    db_pool: sqlx::SqlitePool,
}

impl AppState {
    pub fn new(db_pool: sqlx::SqlitePool) -> Self {
        Self { db_pool }
    }

    pub fn db_pool(&self) -> &sqlx::SqlitePool {
        &self.db_pool
    }
}

pub struct AppState {
    pub pool: sqlx::SqlitePool,
}

impl AppState {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        Self { pool }
    }
}

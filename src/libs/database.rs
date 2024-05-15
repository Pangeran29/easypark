use std::time::Duration;

use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub async fn build() -> Pool<Postgres> {
    let db_conn = std::env::var("DATABASE_URL").expect("DATABASE_URL not defined.");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&db_conn)
        .await
        .expect("Failed to connect to the database.");

    pool
}

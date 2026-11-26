use sqlx::postgres::PgPoolOptions;
pub fn pool() -> sqlx::PgPool { PgPoolOptions::new().max_connections(5).connect_lazy("postgres://localhost/seikika").unwrap() }

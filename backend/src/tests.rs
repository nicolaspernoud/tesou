#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use actix_web::web::Data;
    use diesel_migrations::{EmbeddedMigrations, MigrationHarness};

    use crate::{
        app::AppConfig,
        models::{
            position_tests::position_test, position_ws::WebSocketsState,
            position_ws_tests::position_ws_test, user_tests::user_test,
        },
        token::token_test,
    };
    #[actix_rt::test]
    async fn test_models() {
        use diesel::r2d2::{self, ConnectionManager};
        use diesel::SqliteConnection;
        std::env::set_var("RUST_LOG", "debug");
        env_logger::init();

        // set up database connection pool
        let manager = ConnectionManager::<SqliteConnection>::new("db/test_db.sqlite");
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");
        pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("db/migrations");
        pool.get()
            .expect("couldn't get db connection from pool")
            .run_pending_migrations(MIGRATIONS)
            .expect("couldn't run migrations");

        // Set up authorization token
        let app_config = AppConfig::new("0101".to_string(), "0202".to_string());
        let app_data = Data::new(app_config);
        let ws_state = WebSocketsState {
            index: Mutex::new(0),
            ws_actors: Arc::new(Mutex::new(Vec::new())),
        };
        let ws_state = Data::new(ws_state);

        user_test(&pool, &app_data, &ws_state).await;
        position_test(&pool, &app_data, &ws_state).await;
        token_test(&pool, &app_data, &ws_state).await;
        position_ws_test(&pool, &app_data, &ws_state).await;
    }
}

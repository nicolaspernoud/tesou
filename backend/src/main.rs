#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use std::env;

use actix_web::web::Data;
use actix_web::HttpServer;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};
use positions_server::PositionsServer;
use tokio::{spawn, try_join};

use crate::app::AppConfig;

mod app;
mod db_options;
mod errors;
mod models;
mod positions_handler;
mod positions_server;
mod schema;
#[cfg(test)]
pub mod tester;
#[cfg(test)]
mod tests;
mod token;
mod utils;

use log::info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }

    env_logger::init();

    // create the db folder if it doesn't already exist
    std::fs::create_dir_all("db").expect("failed creating db folder");

    // set up database connection pool
    let manager = ConnectionManager::<SqliteConnection>::new("db/db.sqlite");
    let pool = r2d2::Pool::builder()
        .connection_customizer(Box::new(db_options::ConnectionOptions {
            enable_foreign_keys: false,
            busy_timeout: Some(std::time::Duration::from_secs(30)),
        }))
        .build(manager)
        .expect("failed to create pool.");
    pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("db/migrations");
    pool.get()
        .expect("couldn't get db connection from pool")
        .run_pending_migrations(MIGRATIONS)
        .expect("couldn't run migrations");

    // Set up authorization token
    let app_config = AppConfig::new(
        env::var("TOKEN").unwrap_or_else(|_| -> String {
            let token = crate::utils::random_string();
            info!("Authorization token: {}", token);
            token
        }),
        env::var("API_KEY").expect("Open Cell ID API Key not found"),
    );
    // Data should be constructed outside the HttpServer::new closure if shared, potentially mutable state is desired...
    let app_config = Data::new(app_config);
    let bind = "0.0.0.0:8080";

    // Positions server
    let (positions_server, server_tx) = PositionsServer::new();
    let positions_server = spawn(positions_server.run());

    // Start HTTP server
    let http_server = HttpServer::new(move || create_app!(pool, &app_config, &server_tx))
        .bind(&bind)?
        .run();

    try_join!(http_server, async move { positions_server.await.unwrap() })?;

    Ok(())
}

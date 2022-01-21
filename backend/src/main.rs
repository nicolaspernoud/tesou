#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use std::env;

use actix_web::HttpServer;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

use crate::app::AppConfig;

mod app;
mod errors;
mod models;
mod schema;
#[cfg(test)]
pub mod tester;
#[cfg(test)]
mod tests;
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
        .build(manager)
        .expect("failed to create pool.");
    embed_migrations!("db/migrations");
    embedded_migrations::run_with_output(
        &pool.get().expect("couldn't get db connection from pool"),
        &mut std::io::stdout(),
    )
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

    let bind = "0.0.0.0:8080";

    // Start HTTP server
    HttpServer::new(move || create_app!(pool, app_config.clone()))
        .bind(&bind)?
        .run()
        .await
}

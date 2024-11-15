mod auth;
mod config;
mod controller;
mod db;
mod exception;
mod gitop;
#[macro_use]
mod dev;

use std::{path::Path, process::exit};

use crate::controller::config_controller;
use actix_web::{error, web, App, HttpResponse, HttpServer};
use config::*;
use db::{get_env_var, HOOK_LOCAL_DIR};
use dotenv::dotenv;
use env_logger::Env;
use gitop::{update_repo, write_current_git_info, Repo};
use log::{error, info};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    info!("Hook Api version: {}", env!("CARGO_PKG_VERSION"));

    if std::env::args().nth(1) == Some("init".to_string()) {
        info!("Hook Api Init mode Detected");
        let dir = get_env_var(HOOK_LOCAL_DIR);
        let dirc = dir.clone();
        let repository = Repo::env();
        let res = update_repo(dir, repository).await;
        match res {
            Ok(r) => {
                info!("Hook Api Init mode Success: {:?}", r);
                write_current_git_info(Path::new(&dirc)).expect("Error write commit info");
                exit(0)
            }
            Err(e) => {
                error!("Hook Api Init mode Failed: {:?}", e);
                exit(1)
            }
        }
    }

    let json_config = web::JsonConfig::default()
        .limit(1024)
        .error_handler(|err, _req| {
            error::InternalError::from_response(err, HttpResponse::Conflict().finish()).into()
        });
    //let db_state = DbState::connect().await;
    HttpServer::new(move || {
        App::new()
            //.app_data(web::Data::new(db_state.clone()))
            .app_data(json_config.to_owned())
            .configure(config_controller)
    })
    .workers(1)
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

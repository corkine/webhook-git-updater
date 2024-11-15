use std::{env, ops::Deref, path::Path, sync::Arc};

use actix_web::{http::Error, rt::time, web, FromRequest};
use futures::future::{ready, Ready};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

use crate::{WEB_DB, DATA_DB};

#[derive(Clone)]
pub struct DbState {
    pub data_db: SqlitePool,
    pub web_db: SqlitePool,
}

impl DbState {
    pub async fn connect() -> Arc<DbState> {
        DbState::wait_for_file(DATA_DB).await;
        DbState::wait_for_file(WEB_DB).await;
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(format!("sqlite:{DATA_DB}").as_str())
            .await
            .expect(format!("connect db {} failed", DATA_DB).as_str());
        let pool2 = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(format!("sqlite:{WEB_DB}").as_str())
            .await
            .expect(format!("connect db {} failed", WEB_DB).as_str());
        let db_state = DbState {
            data_db: pool,
            web_db: pool2,
        };
        Arc::new(db_state)
    }
    async fn wait_for_file(path: &str) {
        while !Path::new(path).exists() {
            eprintln!("Waiting for database file: {}", path);
            time::sleep(std::time::Duration::from_secs(5)).await;
        }
    }
}

pub struct DataDb(pub SqlitePool);

pub struct WebDb(pub SqlitePool);

impl Deref for DataDb {
    type Target = SqlitePool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for WebDb {
    type Target = SqlitePool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for DataDb {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let data = req.app_data::<web::Data<Arc<DbState>>>().unwrap();
        ready(Ok(DataDb(data.data_db.clone())))
    }
}

impl FromRequest for WebDb {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let data = req.app_data::<web::Data<Arc<DbState>>>().unwrap();
        ready(Ok(WebDb(data.web_db.clone())))
    }
}

pub const HOOK_LOCAL_DIR: &str = "HOOK_LOCAL_DIR";
pub const HOOK_USER: &str = "HOOK_USER";
pub const HOOK_PASSWORD: &str = "HOOK_PASSWORD";
pub const HOOK_GIT_URL: &str = "HOOK_GIT_URL";
pub const HOOK_GIT_USER: &str = "HOOK_GIT_USER";
pub const HOOK_GIT_PASSWORD: &str = "HOOK_GIT_PASSWORD";
pub const HOOK_GIT_BRANCH: &str = "HOOK_GIT_BRANCH";

pub fn get_env_var(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("{} must be set", key))
}
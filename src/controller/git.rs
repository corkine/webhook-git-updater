use serde_json::json;
use crate::{
    db::{get_env_var, HOOK_LOCAL_DIR},
    exception::ApiError, gitop::{update_repo, Repo},
};
use actix_web::{get, web, Responder, Scope};

#[get("/sync")]
async fn sync_dir() -> Result<impl Responder, ApiError> {
    let dir = get_env_var(HOOK_LOCAL_DIR);
    let repository = Repo::env();
    let res = update_repo(dir, repository).await;
    match res {
        Ok(r) => Ok(
            json!({"message": r, "status": 1})
                .to_string()
                .customize()
                .insert_header(("Content-Type", "application/json")),
        ),
        Err(e) => Err(ApiError::GitOpsError(format!("{:?}", e))),
    }
}

pub fn register() -> Scope {
    web::scope("/git").service(sync_dir)
}

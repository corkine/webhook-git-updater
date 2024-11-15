use std::path::Path;

use crate::{
    db::{get_env_var, HOOK_GIT_BRANCH, HOOK_GIT_PASSWORD, HOOK_GIT_URL, HOOK_GIT_USER, HOOK_LOCAL_DIR},
    exception::ApiError,
};
use actix_web::{get, web, Responder, Scope};
use git2::{AutotagOption, Cred, FetchOptions, RemoteCallbacks, Repository};
use log::{error, info};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct WebhookPayload {
    repository: Repo,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Repo {
    url: String,
    user: String,
    pass: String,
    #[serde(default = "String::default")]
    branch: String,
}

impl Default for Repo {
    fn default() -> Self {
        Self {
            url: String::new(),
            user: String::new(),
            pass: String::new(),
            branch: "master".to_string(),
        }
    }
}

#[get("/sync")]
async fn sync_dir() -> Result<impl Responder, ApiError> {
    let dir = get_env_var(HOOK_LOCAL_DIR);
    let repository = Repo {
        url: get_env_var(HOOK_GIT_URL),
        user: get_env_var(HOOK_GIT_USER),
        pass: get_env_var(HOOK_GIT_PASSWORD),
        branch: get_env_var(HOOK_GIT_BRANCH)
    };
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

use serde_json::json;
use tokio;

fn is_valid_git_repo(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    let git_dir = path.join(".git");
    if !git_dir.is_dir() {
        return false;
    }
    true
}

async fn update_repo(repo_path: String, r: Repo) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&repo_path);

        if !is_valid_git_repo(path) {
            error!("Path is not a valid git repository, may empty or curroptional, cloning...");
            return clone_repo(&path, &r);
        }

        match update_existing_repo(&path, &r) {
            Ok(_) => Ok("Repository updated successfully.".to_string()),
            Err(e) => {
                error!("Failed to update repo: {:?}, try full clone!", e);
                std::fs::remove_dir_all(path)?;
                clone_repo(&path, &r)
            }
        }
    }).await?
}

fn clone_repo(path: &Path, r: &Repo) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    info!("Cloning repository...");
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, _username_from_url, _allowed_types| {
        Cred::userpass_plaintext(&r.user, &r.pass)
    });

    let mut fo = FetchOptions::new();
    fo.remote_callbacks(callbacks);

    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fo);

    builder.clone(&r.url, path)?;

    if !is_valid_git_repo(path) {
        return Err("Failed to create a valid Git repository".into());
    }

    info!("Repository cloned successfully.");
    Ok("Repository cloned successfully.".to_string())
}

fn update_existing_repo(path: &Path, r: &Repo) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let repo = Repository::open(path)?;
    
    // 配置远程仓库
    let mut remote = match repo.find_remote("origin") {
        Ok(remote) => remote,
        Err(_) => repo.remote("origin", &r.url)?,
    };

    // 设置认证回调
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, _username_from_url, _allowed_types| {
        Cred::userpass_plaintext(&r.user, &r.pass)
    });

    // 配置 fetch 选项
    let mut fo = FetchOptions::new();
    fo.remote_callbacks(callbacks);
    fo.download_tags(AutotagOption::All);

    // 执行 fetch
    remote.fetch(&[] as &[&str], Some(&mut fo), None)?;

    // 获取远程分支的最新 commit
    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    let commit = repo.reference_to_annotated_commit(&fetch_head)?;

    // 执行 fast-forward 或 reset
    let mut reference = repo.find_reference(&format!("refs/heads/{}", r.branch))?;
    reference.set_target(commit.id(), "Fast-Forward")?;
    
    // 更新工作目录
    repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;

    info!("Repository updated successfully.");
    Ok(())
}

pub fn register() -> Scope {
    web::scope("/git").service(sync_dir)
}

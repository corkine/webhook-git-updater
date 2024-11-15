use std::path::Path;

use git2::{AutotagOption, Cred, FetchOptions, ObjectType, RemoteCallbacks, Repository};
use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::db::{get_env_var, HOOK_GIT_BRANCH, HOOK_GIT_PASSWORD, HOOK_GIT_URL, HOOK_GIT_USER};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Repo {
    pub url: String,
    pub user: String,
    pub pass: String,
    #[serde(default = "String::default")]
    pub branch: String,
}

impl Repo {
    pub fn env() -> Self {
        Repo {
            url: get_env_var(HOOK_GIT_URL),
            user: get_env_var(HOOK_GIT_USER),
            pass: get_env_var(HOOK_GIT_PASSWORD),
            branch: get_env_var(HOOK_GIT_BRANCH),
        }
    }
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

pub async fn update_repo(
    repo_path: String,
    r: Repo,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
    })
    .await?
}

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

fn clone_repo(path: &Path, r: &Repo) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    info!("Cloning repository...");
    let mut callbacks = RemoteCallbacks::new();
    callbacks.certificate_check(|_, _| true);
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

    let repo = Repository::open(path)?;
    repo.set_head(&format!("refs/heads/{}", r.branch))?;
    repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))?;

    info!("Repository cloned successfully.");
    Ok("Repository cloned successfully.".to_string())
}

fn update_existing_repo(
    path: &Path,
    r: &Repo,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let repo = Repository::open(path)?;

    // 配置远程仓库
    let mut remote = match repo.find_remote("origin") {
        Ok(remote) => remote,
        Err(_) => repo.remote("origin", &r.url)?,
    };

    // 设置认证回调
    let mut callbacks = RemoteCallbacks::new();
    callbacks.certificate_check(|_, _| true);
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

use serde_json::json;
use std::fs::File;
use std::io::Write;

pub fn write_current_git_info(repo_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let repo = Repository::open(repo_path)?;

    let head = repo.head()?;
    let branch_name = head.shorthand().unwrap_or("HEAD detached").to_string();

    let obj = repo.head()?.resolve()?.peel(ObjectType::Commit)?;
    let commit = obj.into_commit().map_err(|_| "Couldn't find commit")?;

    let message = commit.message().unwrap_or("").to_string();
    let author = commit.author().name().unwrap_or("").to_string();
    let email = commit.author().email().unwrap_or("").to_string();
    let timestamp = commit.time().seconds();
    let commit_id = commit.id().to_string();

    let json_data = json!({
        "branch": branch_name,
        "current_commit": {
            "id": commit_id,
            "message": message,
            "author": author,
            "email": email,
            "timestamp": timestamp
        }
    });

    let file_path = repo_path.join("GIT_COMMIT");
    let mut file = File::create(file_path)?;
    file.write_all(serde_json::to_string_pretty(&json_data)?.as_bytes())?;

    Ok(())
}

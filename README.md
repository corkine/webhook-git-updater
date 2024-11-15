# webhook-git-updater

这是一个用于 WebHook 更新 Git 仓库时自动拉取最新代码的 Web 应用程序。

创建 `.env` 文件或从环境变量提供：

```env
HOOK_LOCAL_DIR=repo
HOOK_USER=admin
HOOK_PASSWORD=admin
HOOK_GIT_URL=https://git.gitrepo.com/user/repo_name
HOOK_GIT_BRANCH=cyber-me
HOOK_GIT_USER=corkine
HOOK_GIT_PASSWORD=5fb07a7416462
```

```bash
cargo run --release
# or use container
docker build -t webhook-git-updater:latest .
docker run -it --rm \
    -p 8080:8080 \
    -v ../repo:/app/repo \
    -e HOOK_LOCAL_DIR=repo \
    -e HOOK_USER=admin \
    -e HOOK_PASSWORD=123 \
    -e HOOK_GIT_URL=https://git.abc.com/user/cyberMe \
    -e HOOK_GIT_BRANCH=cyber-me \
    -e HOOK_GIT_USER=corkine \
    -e HOOK_GIT_PASSWORD=5fb07a7416462 \
    localhost/webhook-git-updater:latest
```

## License

MIT

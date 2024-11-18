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

注意，应用支持作为 init-container 使用，只需要传入命令和参数 `/app/webhook-git-updater init` 即可，其检查更新后直接退出，而不是启动 Web 服务器。其将额外在根目录创建 GIT_COMMIT 文件，包含当前代码的提交信息，可用于容器自行实现不依赖 Git 的前提下进行 Webhook 触发和本地代码比对以确定是否要执行滚动更新。

注意，对于文件的修改，如果仓库存在，此工具将强行覆盖并确保和仓库一致，如果不存在，更新时不受影响，但更新错误时重新拉取将导致文件夹被清空。对于配置文件，Kubernetes 环境推荐 ConfigMap 或 Secret 直接挂载为 Volume。

## License

MIT

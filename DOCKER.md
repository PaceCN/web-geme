# Docker 部署测试

Docker 适合用来本地或服务器快速测试整套系统，比每次推 Cloudflare Pages 方便。

## 启动

在 `0.0.1` 目录执行：

```powershell
docker compose up --build
```

打开：

```text
前端游戏：http://localhost:8080
后台管理：http://localhost:8080/admin
后端直连：http://localhost:8787/health
```

默认后台账号：

```text
admin / admin123!
```

## 修改后台账号

正式测试建议先改 `docker-compose.yml`：

```yaml
HUAYUAN_ADMIN_USER: your-admin
HUAYUAN_ADMIN_PASSWORD: a-long-random-password
```

然后重新启动：

```powershell
docker compose up --build
```

## 停止

```powershell
docker compose down
```

## 结构说明

- `Dockerfile.frontend`：构建 Rust WASM 前端，然后用 Nginx 托管 `dist`
- `backend/Dockerfile`：构建并运行 Axum 后端
- `docker/nginx.conf`：前端静态站，同时代理 `/api/` 和 `/admin` 到后端
- `docker-compose.yml`：一键启动前端和后端

## 注意

第一次构建会下载 Rust、Trunk、Nginx、Debian 镜像和 Cargo 依赖，速度取决于网络。

当前后端还是内存数据，容器重启后后台数据会恢复初始状态。正式版需要接 SQLite / PostgreSQL。

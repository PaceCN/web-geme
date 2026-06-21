# 庄园物语后端管理端

这是 Axum 后端骨架，负责后续真实用户数据、广告奖励发放和运营后台管理。

当前前端仍可独立部署到 Cloudflare Pages；`backend/` 不参与 Trunk 静态构建。

## 本地运行

```powershell
cd backend
cargo run
```

默认地址：

```text
http://127.0.0.1:8787/admin
```

本地默认管理员：

```text
账户：admin
密码：admin123!
```

正式部署必须使用环境变量覆盖：

```powershell
$env:HUAYUAN_ADMIN_USER="your-admin"
$env:HUAYUAN_ADMIN_PASSWORD="a-long-random-password"
cargo run
```

## 已包含能力

- `POST /api/admin/login`：后台登录，返回 12 小时有效 token
- `GET /admin`：运营后台页面
- `GET /api/admin/dashboard`：后台看板
- `GET /api/admin/accounts`：后台账户列表
- `GET /api/admin/ad-policy`：查看广告恢复体力配置
- `PUT /api/admin/ad-policy`：修改广告恢复体力配置
- `GET /api/admin/game-switches`：小游戏开关列表
- `POST /api/admin/game-switches/:game_id`：开启/关闭小游戏
- `GET /api/admin/players`：玩家列表
- `POST /api/admin/players/:user_id/stamina`：后台补发体力
- `GET /api/v1/bootstrap/:user_id`：玩家启动数据
- `GET /api/v1/game-switches`：玩家侧小游戏开关
- `POST /api/v1/garden/action`：玩家动作提交，当前支持小游戏结算
- `POST /api/v1/ads/reward`：广告平台回调成功后发放体力奖励

除登录和广告回调外，所有 `/api/admin/*` 接口都需要请求头：

```text
x-admin-token: 登录返回的 token
```

## 广告恢复体力流程

1. 前端调用广告 SDK 展示激励视频。
2. 广告 SDK 返回播放成功和平台凭证。
3. 前端请求 `POST /api/v1/ads/reward`。
4. 后端校验广告位、凭证、每日次数和体力上限。
5. 后端发放体力，并写入审计日志。

当前是内存存储原型，生产版本建议接入 PostgreSQL / SQLite。密码目前使用带固定盐的 SHA-256 原型实现，正式版本应换成 Argon2id / bcrypt，并把 session 存进服务端数据库或 Redis。

## 部署说明

Cloudflare Pages 只负责 `0.0.1` 根目录的静态前端构建，不会自动运行 `backend/`。

后端需要单独部署到支持 Rust 服务的环境，例如 VPS、容器平台、Fly.io、Railway、Render，或后续改写成 Cloudflare Workers。前端接真实后端时，再把 `src/main.rs` 里的本地 `run_action` 替换成请求这些接口。

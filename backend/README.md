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

## 已包含能力

- `GET /admin`：运营后台页面
- `GET /api/admin/dashboard`：后台看板
- `GET /api/admin/ad-policy`：查看广告恢复体力配置
- `PUT /api/admin/ad-policy`：修改广告恢复体力配置
- `GET /api/admin/players`：玩家列表
- `POST /api/admin/players/:user_id/stamina`：后台补发体力
- `POST /api/v1/ads/reward`：广告平台回调成功后发放体力奖励

## 广告恢复体力流程

1. 前端调用广告 SDK 展示激励视频。
2. 广告 SDK 返回播放成功和平台凭证。
3. 前端请求 `POST /api/v1/ads/reward`。
4. 后端校验广告位、凭证、每日次数和体力上限。
5. 后端发放体力，并写入审计日志。

当前是内存存储原型，生产版本建议接入 PostgreSQL / SQLite，并给 `/api/admin/*` 增加管理员登录鉴权。

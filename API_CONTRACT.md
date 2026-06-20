# API Contract

后续 Rust Axum 后端建议提供玩家接口、广告奖励接口和管理端接口。`backend/` 目录已经给出 Axum 原型。

## GET /api/v1/bootstrap

返回玩家当前完整状态：

```json
{
  "player": {},
  "inventory": [],
  "orders": [],
  "tasks": [],
  "transactions": []
}
```

## POST /api/v1/garden/action

请求：

```json
{
  "actionId": "submit_order",
  "targetId": "order_001",
  "payload": {}
}
```

响应：

```json
{
  "success": true,
  "message": "交货结账成功",
  "state": {}
}
```

## 当前前端动作

```text
purchase_item
drink_elixir
watch_ad_restore
submit_order
complete_task_action
claim_task
buy_bazaar
sell_bazaar
finish_minigame
```

正式联机版本中，金币、体力、背包、订单、任务进度、小游戏结算都应该由后端校验并落库。

## POST /api/v1/ads/reward

激励广告播放完成后，由前端携带广告平台回调凭证请求后端发放体力。

请求：

```json
{
  "user_id": "u_10001",
  "placement_id": "reward_stamina",
  "ad_network": "demo",
  "proof_token": "server_callback_token"
}
```

响应：

```json
{
  "accepted": true,
  "message": "广告奖励已发放",
  "current_stamina": 20,
  "max_stamina": 30,
  "claims_left_today": 5
}
```

## 管理端接口

```text
GET  /admin
GET  /api/admin/dashboard
GET  /api/admin/ad-policy
PUT  /api/admin/ad-policy
GET  /api/admin/players
POST /api/admin/players/:user_id/stamina
```

生产版本必须给 `/api/admin/*` 增加管理员登录鉴权和操作审计落库。

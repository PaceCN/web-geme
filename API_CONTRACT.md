# API Contract

后续 Rust Axum 后端建议提供两个核心接口。

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
submit_order
complete_task_action
claim_task
buy_bazaar
sell_bazaar
finish_minigame
```

正式联机版本中，金币、体力、背包、订单、任务进度、小游戏结算都应该由后端校验并落库。

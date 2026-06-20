# models 目录规划

当前版本为了快速部署测试，主要数据结构仍集中在 `src/main.rs` 中。后续接入后端或继续拆分游戏模块时，建议把共享模型逐步迁移到本目录。

建议模型：

```text
PlayerModel              玩家基础状态：等级、经验、金币、体力、小游戏最高分
InventoryItemModel       背包物品：item_id、名称、类型、数量
OrderModel               居民契约订单：需求、奖励、状态
TaskModel                庄园内务任务：进度、目标、奖励状态
GameModuleMeta           游戏模块元信息：模块 ID、名称、入口文案、奖励说明
GameRunResult            小游戏结算结果：分数、材料奖励、经验奖励
GameActionRequest        前端动作请求：动作类型、参数、客户端状态版本
GameActionResponse       后端动作响应：新状态、提示消息、奖励明细
```

拆分原则：

```text
models/ 只放数据结构和序列化定义
modules/ 放小游戏内部逻辑
main.rs 保留界面组装、状态信号和动作分发
```

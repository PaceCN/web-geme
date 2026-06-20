# models 目录说明

当前版本已经把主框架共享数据结构迁移到 `src/models/mod.rs`。这里放前端、小游戏模块、后端接口都需要认识的数据模型；不要放界面组件和小游戏内部规则。

当前已包含：

```text
Player                  玩家基础状态：等级、经验、金币、体力、小游戏最高分、广告体力次数
InventoryItem           背包物品：item_id、名称、类型、数量
Requirement             订单材料需求
Order                   居民契约订单：需求、奖励、状态
Task                    庄园内务任务：进度、目标、奖励状态
GameState               前端本地完整状态
GameReward              小游戏奖励项
GameRunResult           小游戏结算结果
AdRewardPolicy          广告恢复体力策略
```

拆分原则：

```text
models/ 只放数据结构和序列化定义
modules/ 放小游戏内部逻辑
main.rs 保留界面组装、状态信号和动作分发
backend/ 放真实后端、管理端、广告发奖校验
```

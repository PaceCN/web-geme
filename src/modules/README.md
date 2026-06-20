# modules 目录规划

后续新增小游戏模块时，统一放在本目录。当前 2048 已经迁移到 `src/modules/game_2048.rs`。

```text
modules/game_2048.rs
modules/match3.rs
modules/make10.rs
```

每个小游戏模块建议提供统一能力：

```text
GameState struct            模块内部棋盘 / 分数 / 步数等状态
new()                       创建新局
handle_input / move_dir     处理点击、滑动、拖拽等输入
Overlay component           独立渲染弹窗或玩法界面
finish_score signal         向主框架返回结算分数
```

这样主界面只需要关心“打开游戏、接收结算、发放奖励”，不需要知道每个小游戏的内部规则。

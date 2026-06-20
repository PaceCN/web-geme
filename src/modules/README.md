# modules 目录规划

后续新增小游戏模块时，建议放在本目录。当前版本的 2048 逻辑仍在 `src/main.rs`，后续可以迁移为：

```text
modules/game_2048.rs
modules/match3.rs
modules/make10.rs
```

每个小游戏模块建议提供统一接口：

```text
new_game()                  创建新局
handle_input(input)         处理点击、滑动、拖拽等输入
is_finished()               判断是否结束
settle() -> GameRunResult   生成奖励结算结果
```

这样主界面只需要关心“打开游戏、接收结算、发放奖励”，不需要知道每个小游戏的内部规则。

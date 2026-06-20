# 庄园物语素材清单

所有占位素材位于 `public/assets/`。Cloudflare 构建后会随 `public/` 一起复制到 `dist/public/assets/`。后期替换素材时，建议保持相同分辨率、文件名和路径，避免再改代码或 CSS。

| 模块 | 用途 | 素材分辨率大小 | 素材路径 |
| --- | --- | --- | --- |
| 主界面 | 庄园顶部氛围背景 | 960x360 | `public/assets/ui/bg-garden-terrace.svg` |
| 主界面 | 卡片纸张纹理 | 512x512 | `public/assets/ui/paper-texture.svg` |
| 居民契约 | 契约公告栏插画 | 640x320 | `public/assets/ui/contract-board.svg` |
| 庄园内务 | 清扫小游戏插画 | 640x320 | `public/assets/ui/chore-minigames.svg` |
| 物资背包 | 仓库储藏室插画 | 640x320 | `public/assets/ui/backpack-room.svg` |
| 集市商店 | 小镇商铺插画 | 640x320 | `public/assets/ui/market-stall.svg` |
| 个人信息 | 庄园护照插画 | 640x320 | `public/assets/ui/profile-passport.svg` |
| 2048 | 2048 棋盘底图参考 | 512x512 | `public/assets/games/board-2048.svg` |
| 道具 | 黄金神水 | 256x256 | `public/assets/items/elixir-water.svg` |
| 道具 | 庄园能量饮料 | 256x256 | `public/assets/items/potion-energy.svg` |
| 道具 | 庄园荣誉徽章 | 256x256 | `public/assets/items/estate-badge.svg` |
| 材料 | 干枯杂草 | 256x256 | `public/assets/items/debris-weed.svg` |
| 材料 | 金色落叶 | 256x256 | `public/assets/items/debris-leaves.svg` |
| 材料 | 修剪碎木 | 256x256 | `public/assets/items/estate-wood.svg` |
| 材料 | 坚韧蛛丝 | 256x256 | `public/assets/items/spider-silk.svg` |

## 替换建议

- UI 横幅类素材建议保持 `640x320` 或 `960x360`，适合移动端卡片裁切。
- 道具图标建议保持 `256x256` 正方形，透明背景 PNG/WebP 也可以，但路径和扩展名变更后需要同步修改 CSS 或代码。
- 2048 棋盘是界面参考素材，当前真实棋盘由 CSS 网格绘制，保证交互和数字渲染稳定。

# 庄园物语 Rust Web Demo

这版是按 `开发文档.txt` 和原始 Android Compose 项目内容重建的 Rust + WebAssembly 前端工程。

## 已还原的原始项目内容

- 顶部状态栏：昵称、庄园等阶、金币、体力、经验
- 底部 5 个主导航：居民契约、庄园内务、物资背包、集市商店、个人信息
- 居民契约：5 个初始订单、材料校验、金币和 EXP 奖励
- 庄园内务：三个小游戏入口、五个常规清扫任务、任务进度和领奖
- 物资背包：药剂、徽章、废料材料、药剂使用恢复体力
- 集市商店：老皮特官方商店、玩家市集买入、背包材料挂牌出售并扣 10% 手续费
- 个人信息：账号卡、小游戏最高分、金币和体力总览
- 2048：Rust 状态实现，支持手机滑动
- Canvas 2D：庄园场景占位和 Hotspot 点击跳转

## 技术结构

```text
0.0.1/
├── Cargo.toml              # Rust / Leptos / wasm-bindgen 依赖
├── Trunk.toml              # Trunk 构建配置
├── index.html              # Trunk 入口
├── styles.css              # 手机竖屏样式
├── _headers                # Cloudflare Pages 响应头
├── _redirects              # Cloudflare Pages SPA 回退
├── src/main.rs             # Leptos 应用和游戏逻辑
└── public/                 # PWA、asset_manifest、Cloudflare Pages 文件
```

## 本地运行

需要先安装 Rust 和 Trunk：

```powershell
rustup target add wasm32-unknown-unknown
cargo install trunk
trunk serve --open
```

当前这台环境没有 `cargo/rustc`，所以我没法在本机完成编译验证。

## Cloudflare Pages

Cloudflare Pages 可以这样配：

```text
Project root: 留空 / 仓库根目录
Build command: bash scripts/cloudflare-pages-build.sh
Build output directory: dist
```

如果你用 `sync-github.cmd` 上传时选择的是 `0.0.1 目录`，那么 GitHub 仓库根目录已经是本项目目录，Cloudflare Pages 的 `Project root` 应该留空。

如果你上传的是整个工作区，并且 GitHub 仓库里还能看到外层的 `0.0.1/` 文件夹，那么 `Project root` 才填写 `0.0.1`。

注意不要把输出目录配置成 `0.0.1` 或仓库根目录。`index.html` 是 Trunk 源入口，只有构建后的 `dist/index.html` 才会自动注入 WASM/JS 脚本。

Cloudflare Pages 默认构建镜像里可能没有 `rustup`。`scripts/cloudflare-pages-build.sh` 会先安装 Rust stable，再添加 `wasm32-unknown-unknown` 目标、安装 Trunk，并执行 `trunk build --release`。

如果部署后 F12 看到 `Manifest: Line: 1, column: 1, Syntax error.`，通常是 manifest 请求被 SPA 回退成了 `index.html`。这一版已经把 manifest 改到 `public/manifest.webmanifest`，并补了 Cloudflare 的 manifest content-type。

如果 Cloudflare 的构建镜像没有预装 Rust，建议先在本地或 CI 构建 `dist`，再把 `dist` 作为静态站发布。

## 后端接入

当前前端是本地状态版本，逻辑在 `src/main.rs`。后续 Axum 后端稳定后，可以把 `run_action` 和初始化状态替换为：

```text
GET  /api/v1/bootstrap
POST /api/v1/garden/action
```

接口建议见 `API_CONTRACT.md`。

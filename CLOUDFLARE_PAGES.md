# Cloudflare Pages 部署说明

当前项目是 Rust + Leptos + Trunk 的 WASM 前端。Cloudflare Pages 默认构建环境没有 `rustup`，所以不能直接使用：

```text
rustup target add wasm32-unknown-unknown && cargo install trunk && trunk build --release
```

如果日志里出现：

```text
/bin/sh: 1: rustup: not found
```

说明 Cloudflare Pages 仍然在使用旧的 Build command。

## 正确配置

如果 GitHub 仓库根目录就是本项目，也就是仓库里直接能看到：

```text
Cargo.toml
Trunk.toml
index.html
scripts/cloudflare-pages-build.sh
src/
public/
```

Cloudflare Pages 设置为：

```text
Root directory: 留空
Build command: bash scripts/cloudflare-pages-build.sh
Build output directory: dist
```

如果 GitHub 仓库根目录外面还有一层 `0.0.1/` 文件夹，则设置为：

```text
Root directory: 0.0.1
Build command: bash scripts/cloudflare-pages-build.sh
Build output directory: dist
```

## 为什么必须改 Build command

项目里的 `scripts/cloudflare-pages-build.sh` 会做这些事：

```text
安装 Rust stable
添加 wasm32-unknown-unknown target
安装 Trunk
执行 trunk build --release
```

Cloudflare 当前日志显示它还在执行旧命令，所以还没有走到这个脚本。

## 修改位置

在 Cloudflare Dashboard 中进入：

```text
Workers & Pages
选择你的 Pages 项目
Settings
Builds & deployments
Build configuration
```

把 Build command 改为：

```text
bash scripts/cloudflare-pages-build.sh
```

然后重新部署。

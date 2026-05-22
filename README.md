# Windows Optimization Tool

一个从 Electron 迁移到 Tauri 的轻量级 Windows 桌面工具箱。

## 技术栈

前端：

```text
Vue 3 + Naive UI + Vite
```

后端：

```text
Tauri 2 + Rust
```

前端负责界面、组件和交互逻辑；Rust 后端负责本地能力，例如启动本地工具、读写文件、调用命令行等。

## 环境准备

需要安装：

```text
Node.js / npm
Rust / Cargo
Visual Studio Build Tools C++ 工具链
```

本机已安装并验证：

```bash
cargo --version
rustc --version
npm --version
```

如果换电脑开发，建议安装 Rust：

```text
https://www.rust-lang.org/tools/install
```

Windows 编译还需要 Visual Studio Build Tools，并选择 C++ build tools。

## 安装依赖

```bash
npm install
```

## 开发启动

```bash
npm run dev
```

等价：

```bash
npm start
```

项目脚本会先把 %USERPROFILE%\.cargo\bin 加到当前命令的 PATH，避免新 PowerShell 找不到 cargo。

开发模式会自动打开 WebView DevTools，前端 console.log 可以在 DevTools Console 中查看。

Tauri CLI 会自动启动：

```text
Vite dev server: http://127.0.0.1:1420
Tauri desktop window: 系统工具箱
```

## 只构建前端

```bash
npm run build:web
```

前端产物输出到：

```text
dist/
```

## 打包桌面应用

```bash
npm run build
```

等价：

```bash
npm run pack:win
```

当前已验证可成功构建。

主要输出：

```text
src-tauri/target/release/system_toolbox_tauri.exe
src-tauri/target/release/bundle/nsis/系统工具箱_1.0.0_x64-setup.exe
```

当前本机产物大小约：

```text
system_toolbox_tauri.exe: 9 MB
NSIS setup: 6.4 MB
```

## 项目结构

```text
src/
  App.vue
  main.js
  styles.css
  components/toolbox/
  views/DashboardPage.vue
src-tauri/
  src/lib.rs
  src/main.rs
  tauri.conf.json
  resources/tools/
```

## 本地工具启动机制

前端通过 Tauri invoke 调用 Rust 命令：

```js
invoke('open_bundled_tool', {
  toolKey: 'windowsActivation',
})
```

Rust 后端只允许打开白名单中的工具，避免前端传入任意本地路径执行。

工具资源目录：

```text
src-tauri/resources/tools/
```

注意：请只放置你有权分发和使用的合法工具。本仓库不会提交或分发可能涉及授权绕过的二进制工具。

## GitHub 提交说明

以下目录不会提交：

```text
node_modules/
dist/
src-tauri/target/
```

本地敏感/授权类工具二进制也不会提交：

```text
src-tauri/resources/tools/windows-activation/windowsActivation.exe
```



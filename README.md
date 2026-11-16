# 回声 · 微信回复助手

macOS 快捷浮窗回复工具，使用 Tauri 2、Vue 3、TypeScript 和 Rust 构建。**当前为 v0.1.0 页面体验版，默认使用虚构私聊与群聊；真实微信自动读取和 AI 自动画像尚未接入。**

已实现模型配置与兼容 Chat Completions 的请求代码，尚未完成真实服务商联调。未配置时显示明确标记的演示候选，不代表真实 AI 结果。代码公开不等于完整 MVP 已完成。

## 已有功能

- 私聊回复、改写和主动跟进；候选编辑、复制，发送由用户完成。
- 群聊示例中选择消息和回复对象；群聊不建立或使用画像。
- 在设置中新增、编辑、删除语气模板；启用的模板才显示在主面板。
- 本次背景默认关闭；开启才显示输入框，关闭不传入生成请求。
- 独立聊天记录视图、手动刷新、可配置 5～300 秒的自动刷新。**当前刷新只重载示例，不能同步真实微信消息。** 窗口隐藏、生成中或使用手动片段时暂停，保留草稿。
- 私聊沟通档案、来源查看、手动补充、示例观察及冲突核对；桌面 SQLite 持久化。
- Option + Space 唤起、托盘、置顶和原生剪贴板。部分原生场景仍待完整验收。

## 本地运行

已验证环境：Apple silicon macOS 26.6.2、Node.js 24.14.0、Rust 1.100.0。构建配置最低系统为 macOS 14，尚未验证最低版本和 Intel。需要 Xcode Command Line Tools；工具链由 `.nvmrc` 和 `rust-toolchain.toml` 固定。

```bash
npm ci
npm run dev
```

浏览器访问 `http://127.0.0.1:1420`。浏览器只提供示例交互，不能设置模型 Key 或使用原生能力。

```bash
# 若 Cargo 不在 PATH，先执行此行
export PATH="$HOME/.cargo/bin:$PATH"

# 原生开发
npm run tauri -- dev

# 本机体验包，不是已公证的对外安装包
npm run tauri -- build --debug --bundles app
```

完成打包后，双击 `打开回声.command`，或打开 `src-tauri/target/debug/bundle/macos/回声.app`。首次克隆不包含编译产物，需要先构建。更新本机包后应退出旧版再重新打开。

## 自测

```bash
npm run test
npm run build
npm run lint
npm run format:check
cargo test --locked --manifest-path src-tauri/Cargo.toml
cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

测试使用虚构内容和临时数据库，不访问真实微信，也不调用收费模型。页面交互由 Playwright CLI 单独验证；当前没有 `npm run test:e2e` 脚本。检查范围与未完成项见 [验证记录](docs/VALIDATION.md)。

## 模型与数据

桌面设置中填写 HTTPS 模型地址、模型名和 API Key，并确认外发范围。API Key 仅交给 macOS Keychain；应用库仅保存配置和钥匙串引用，已有 Key 不回传前端。请勿将 Key 填入源码、URL、共享文件或提交到 Git。

HTTP 客户端在应用启动时创建并复用连接池；每次请求仍有 45 秒超时，可取消生成。实际连接能否复用由服务端决定，尚未实测延迟收益和长时间内存趋势。

桌面档案保存在应用数据目录；本次背景、聊天窗口、候选与草稿默认不持久化。应用 SQLite 不是数据库级加密。仓库只包含虚构样本；本机模型配置、数据库、真实聊天、采集工具和历史探测报告均不发布。现版本不会操作微信发送、采集密钥、改签微信或更改 SIP。

## 文档

- [产品需求](docs/PRD.md)
- [技术架构](docs/ARCHITECTURE.md)
- [数据库设计](docs/DB_DESIGN.md)
- [开发计划](docs/PLAN.md)
- [实现状态](docs/DEV_STATUS.md)
- [项目规则](AGENTS.md)

设计文档包含后续目标，实际能力以实现状态和验证记录为准。本仓库不包含上游微信查询程序或其二进制；后续集成须先核验许可、版本、数据契约及权限。

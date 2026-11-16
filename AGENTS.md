# 微信回复助手项目规则

## 1. 当前阶段与文档入口

当前为 v0.1.0 macOS 桌面体验版，使用虚构私聊与群聊。真实微信 ChatSource、新消息增量、真实模型联调和 AI 自动画像尚未完成。不能把演示候选当真实模型结果。

进入项目先读本文件，再按任务读取：

- [docs/PRD.md](docs/PRD.md)：用户故事、功能、边界、验收。
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)：技术选型、模块与接口。
- [docs/DB_DESIGN.md](docs/DB_DESIGN.md)：应用 SQLite 设计。
- [docs/PLAN.md](docs/PLAN.md)：M0 → MVP → 后续迭代。
- [docs/DEV_STATUS.md](docs/DEV_STATUS.md)：v0.1 实现状态、实际验证及剩余限制。
- [docs/VALIDATION.md](docs/VALIDATION.md)：公开自测范围及未完成项；不包含本机真实聊天或凭据。

技术文档冲突时先指出并统一；用户最新明确指令优先。不把原型固定文案当真实模型结果。

## 2. 技术栈与依赖边界

- 目标平台：macOS、Apple silicon；先验收当前机器，不擅自承诺 Intel 或其他系统。
- 桌面：Tauri 2 + Rust stable；UI：Vue 3 + TypeScript strict + Vite + Pinia。
- 工具：Node.js 24、npm workspaces；初始化时固定具体版本并提交锁文件，不擅自升级主版本或换包管理器。
- 原生：Tokio、serde、reqwest；数据：SQLite/rusqlite bundled；模型 Key：macOS Keychain。
- 当前数据源：shared 中虚构 fixture；目标为 Rust ChatSource 封装的只读数据库查询，接入前核实上游版本、许可证、JSON 契约和媒体缓存副作用。
- 正式运行不依赖 Python MCP 服务或本地 HTTP 后端。
- 不直接改上游源码；确需定制时先写明缺口、影响和最小改动范围，不复制整个上游进来重做。

## 3. 目标目录职责

```text
docs/                         产品、架构、数据库、计划
frontend/src/features/reply/   回复对象、语气、上下文、候选与编辑
frontend/src/features/profile/ 沟通档案、观察、核对与历史
frontend/src/features/settings/配置、诊断与清理
frontend/src/bridge/           Tauri 命令/事件唯一调用入口
frontend/src/stores/           UI 状态，不保存秘密
frontend/src/components/       可复用小组件
src-tauri/src/commands/        参数校验和应用服务入口
src-tauri/src/application/     上下文、回复与档案用例
src-tauri/src/domain/          实体、规则与接口
src-tauri/src/infrastructure/  数据源、模型、存储、系统能力
src-tauri/migrations/          应用库增量迁移
src-tauri/binaries/            经校验的目标平台查询程序
scripts/                      构建/校验脚本
```

主要结构已经创建；binaries/、正式微信适配器与部分后续测试目录尚未建立。shared/demo-workspace.json 是前后端共用的虚构会话及资料；不写入真实聊天。前端按功能模块拆分，禁止把所有组件和逻辑塞入 App.vue、App.tsx、index.tsx 或单个 store；App.vue 只组装。前端不得直接拼 SQL、调用任意 shell 或处理微信密钥。

## 4. 构建与测试命令

### 4.1 桌面工程命令

已建立下面的构建脚本；其中 test:e2e 仍是后续约定，尚未创建该 npm 脚本。根目录 npm workspace 管理 frontend，只有一个根 package-lock.json；Tauri CLI 从根目录运行。若 shell 没有默认加入 Cargo 路径，Rust/Tauri 命令需在本次进程设置 `PATH="$HOME/.cargo/bin:$PATH"`，不要据此错误报告“Rust 未安装”。工具链固定为已有的 Rust 1.100.0、Node 24.14.0。

| 命令 | 约定作用 |
| --- | --- |
| npm ci | 按已有锁文件安装依赖；首次初始化生成锁文件时另用 npm install |
| npm run dev | frontend 的 Vite 浏览器态开发；真实 Tauri 能力需 mock bridge |
| npm run tauri -- dev | 启动 macOS 原生开发应用 |
| npm run typecheck | frontend 的 vue-tsc 类型检查 |
| npm run lint | 前端 ESLint 检查 |
| npm run format:check | Prettier 格式检查 |
| npm run test | Vitest 一次性运行，不进入 watch |
| npm run test:e2e | 尚未提供；本轮浏览器验证使用 Playwright CLI，不冒充已有标准测试脚本 |
| npm run build | 前端类型检查与 Vite 构建，不等于原生应用构建 |
| cargo fmt --manifest-path src-tauri/Cargo.toml -- --check | Rust 格式检查 |
| cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings | Rust 静态检查 |
| cargo test --locked --manifest-path src-tauri/Cargo.toml | 规则、仓储及适配器契约测试，不连接真实微信 |
| npm run tauri -- build --target aarch64-apple-darwin | 构建原生个人测试包，另需 sidecar 和签名配置 |
| npm run tauri -- build --debug --bundles app | 当前已用命令，生成本机 arm64 体验版，未捆绑微信查询程序 |

已验证命令与测试数量记录在 DEV_STATUS；尚未实现的脚本不能当作检查通过。按改动选择相关检查，不重复跑已通过且不受影响的昂贵检查。双击根目录“打开回声.command”可启动已打包的应用，不需要启动 Vite。

原生快捷键、置顶、剪贴板、Keychain、实际查询和签名后 app 必须实机验证；浏览器 mock 测试不能替代。真实微信/模型联调不作为普通单元测试默认步骤。

## 5. 编码规范

- 先检查项目、Git 状态（若已初始化）、已有改动及调用范围；保留用户改动，不自动提交。
- 新增/修改方法写有用注释，说明职责、输入边界或副作用；复杂逻辑解释原因，不写逐行翻译。
- TypeScript/JavaScript 的 if、else、for、while、do/while 语句体必须使用花括号，即使只有一行。Rust 遵循标准块语法；Python 使用其缩进语法。
- TypeScript 开启 strict，边界响应先做校验，禁止用 any 掩盖关键模型/数据源类型错误。
- Vue 使用 Composition API 与 `<script setup lang="ts">`，props/events 类型明确；组合函数与展示组件职责分开。
- 模型设置页只临时接收用户本次输入的 API Key 并交给原生保存，不进入 Pinia、localStorage、调试输出；已有 Key 不从原生读回前端。
- Rust 模块按职责拆分；可预期错误返回标准错误，禁止在数据源/网络路径用 unwrap 制造崩溃；不把凭据装进可 Debug 输出的结构。
- IPC DTO 统一 camelCase，Rust 内部 snake_case；由 bridge 统一封装，命令名不散落组件。
- 模型调用、子进程与数据库任务不阻塞 UI 线程；模型网络请求在写事务外执行。
- 保持性能意识：按会话分页、限制上下文与摘录长度，隐藏窗口不扫描；没有测量不引入全量缓存、向量库、任务平台或复杂连接池。
- 只处理实际可发生的问题。若防护需要增加明显复杂度，先说明出错场景、概率和代码成本，得到用户同意后再扩大范围。
- 测试聚焦方向/顺序、己方最后发言、锁定冲突、证据归属、有效期、删除排除、联系人切换与取消等真实规则；不为静态样式堆测试。
- 自动测试用虚构/脱敏样本；测试失败先定位第一个真实错误，不能靠删除关键断言或吞异常让检查变绿。
- 结构定义、调用链和影响分析，若有健康 CodeGraph 索引优先使用；字符串、SQL、配置和日志用 rg。未初始化时不擅自建索引。

## 6. 数据与功能禁止事项

1. 不向微信源库写入，不修改源库索引/表，不调用上游发送、快照、恢复、全量记忆导出等非 MVP 查询能力。
2. 不自动发送、粘贴、模拟回车；“复制成功”不能记成“发送成功”。
3. 不自动改签/降级微信、关闭 SIP、提权采集密钥或恢复旁白。凭据设置与日常查询隔离，凭据采集不作为测试入口。
4. 不把 OCR、旁白或 AX 作为正式主读取链路；不把手动粘贴冒充自动读取。
5. 不通过昵称猜本人或用备注作数据主键；账号、会话或身份不明确时停止相关任务。
6. 不向模型发送其他联系人消息、全量历史、密钥、绝对媒体路径或无关资料。首次外发前确认服务商和范围，端点变更需重新确认。
7. 不默认保存完整聊天、候选、草稿、本次背景、原始 prompt/模型响应；日志只含必要元数据和标准错误码。
8. API Key、微信密钥、管理员密码、原始客户内容不得写入仓库、文档、fixtures、日志或回复；不复制全局规则中的其他系统凭据到本项目。
9. 不静默覆盖手动资料、用户确认内容或锁定资料；未确认观察不参与生成。删除时同步处理证据、提议和正文历史。
10. 不把临时性状态写成永久性格，不捏造关系、事实或承诺；不显示没有校准依据的画像分数。
11. 用户已授权群聊辅助回复：群聊不创建或使用群/成员画像，必须明确选中要回复的消息。当前仅示例群聊；真实群读取仍待接入。不扩展到其他平台、云同步、自动联系人跟随或 MCP 发送工具。
12. 不自动 Git 初始化、提交、推送、发布或签名对外分发；这些操作按用户明确指令执行。

## 7. 完成与报告标准

- 用中文直接说明结果，报告修改文件、验证范围、真实失败及未完成项。
- 区分“文档设计”“静态检查”“前端 mock 测试”“原生运行”“真实读取”“真实模型调用”和“打包后验证”。编译成功不能叫运行通过。
- 技术栈、目录、命令、接口或存储模型发生变化，更新对应文档，不能仅在聊天里解释。
- 本机样本通过不代表所有微信/macOS 版本兼容；不做“零风控”“完全实时”“加密存储”等缺少证据的宣传。

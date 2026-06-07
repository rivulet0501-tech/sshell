# sshell 详细设计文档（MVP）

**版本**：v0.1  
**日期**：2026-06-07  
**状态**：草稿（与 `docs/proposal.md` 对齐）

---

## 1. 设计范围与约束

### 1.1 范围

本详细设计**仅覆盖需求文档 2.1（MVP）**：

1. 建立 SSH 连接（命令行参数 + 运行时新建 Tab）。
2. 断开连接（关闭 Tab / 退出程序）。
3. 多 Tab 管理（新建、关闭、切换、标题显示）。
4. 交互式终端（PTY + ANSI/VT + 全屏程序支持）。
5. 标准化错误码与 stderr 错误输出。

### 1.2 不在本次设计范围

以下能力明确不纳入本设计实现范围：会话保存/快速重连、SFTP/SCP、密钥认证、一次性命令执行、会话日志审计、图形 UI、IPv6、Qt 嵌入终端。

### 1.3 平台与边界条件

- Windows：最低 **Build 17763+**（ConPTY 可用前提）。
- Linux：Ubuntu 20.04+（内核 5.4+）。
- macOS：macOS 15 (Sequoia)。
- 终端类型统一上报：`xterm-256color`。
- 主机指纹策略：MVP 默认信任，不做校验。

---

## 2. 总体架构

### 2.1 架构分层

按“协议层与界面层分离”原则拆分为六层：

1. **入口层（CLI）**：参数解析、初始连接串收集。
2. **应用编排层（App Core）**：生命周期、事件循环、退出码聚合。
3. **会话管理层（Session Manager）**：Tab 与 SSH 会话一一对应，负责创建/切换/关闭。
4. **协议与 I/O 层（SSH + PTY Adapter）**：SSH 通道、数据读写、窗口尺寸同步。
5. **终端渲染层（Terminal Engine）**：ANSI/VT 解释与缓冲区模型。
6. **界面层（TUI）**：Tab 栏、状态栏、输入捕获与快捷键调度。

### 2.2 模块独立性原则

- 每个模块提供稳定接口（trait/抽象接口），上层仅依赖接口，不依赖具体实现。
- 模块内部可单测；跨模块行为通过集成测试验证。
- 平台差异（Unix PTY / Windows ConPTY）封装在统一适配层，避免向上层泄漏条件分支。

---

## 3. 模块详细设计

### 3.1 `cli` 模块（启动参数入口）

#### 职责

- 解析命令行：`sshell <连接串> [<连接串> ...]`
- 输出参数错误（退出码 2）与帮助信息。

#### 输入/输出

- 输入：`argv`
- 输出：`Vec<ConnectionSpec>` 或 `CliError`

#### 可测试性

- 表驱动测试：合法/非法连接串、缺参、端口边界。
- 与其它模块独立，可纯函数测试。

---

### 3.2 `connection_spec` 模块（连接串解析与校验）

#### 职责

- 解析 `用户名:密码@主机[:端口]`。
- 约束：仅 IPv4 或域名；默认端口 22；拒绝 IPv6 字面量。

#### 核心数据结构

- `ConnectionSpec { username, password, host, port }`
- `DisplayName { user_at_host_port }`（用于 Tab 标题）

#### 错误映射

- 格式错误 → 退出码 2（参数错误）。

#### 可测试性

- 仅字符串解析逻辑，独立单测。
- 覆盖 `@`、`:` 多重分隔边界与端口缺省逻辑。

---

### 3.3 `session_manager` 模块（Tab/会话管理）

#### 职责

- 维护 `SessionId -> SessionRuntime` 映射。
- 新建、关闭、激活、前后切换 Tab。
- 维护当前激活索引与 Tab 顺序。

#### 核心数据结构

- `SessionId(u64)`
- `SessionState { id, title, status, created_at, last_error }`
- `SessionManager { sessions, active_id, order }`

#### 关键行为

1. 启动时按 CLI 输入逐个创建会话。
2. 运行时接收“新建连接串”命令并新增 Tab。
3. 关闭当前 Tab 时自动选中相邻 Tab。
4. 最后一个 Tab 关闭时触发程序退出流程。

#### 可测试性

- 纯内存状态机测试（无需 SSH/终端）。
- 覆盖并发创建、连续关闭、边界切换（首尾循环）。

---

### 3.4 `ssh_runtime` 模块（SSH 会话执行）

#### 职责

- 建立 SSH 连接与认证（用户名+密码）。
- 创建远端交互式 shell channel。
- 双向转发：本地输入 → 远端，远端输出 → 终端引擎。

#### 接口抽象

- `SshClient` trait：`connect/auth/open_shell/resize/write/read/close`
- 真实实现：基于选定 SSH 库
- 测试实现：`MockSshClient`

#### 错误映射

- 网络错误 → 3
- 认证失败 → 4
- SSH 协议错误 → 5

#### 可测试性

- 使用 mock 注入认证失败/断线/超时。
- 与 TUI 解耦，可独立进行行为测试。

---

### 3.5 `pty_adapter` 模块（平台终端适配）

#### 职责

- Unix 使用 POSIX PTY，Windows 使用 ConPTY。
- 统一暴露 `spawn/read/write/resize/close` 能力。
- 向上层屏蔽平台差异。

#### 平台策略

- 编译期条件分支：`cfg(unix)` / `cfg(windows)`。
- Windows 启动时校验 Build 号，低于 17763 则直接报终端初始化失败（退出码 6）。

#### 错误映射

- PTY/ConPTY 创建失败 → 6。

#### 可测试性

- 接口层使用假实现验证上层逻辑。
- 平台实现在对应平台做集成测试。

---

### 3.6 `terminal_engine` 模块（终端仿真）

#### 职责

- 处理 ANSI/VT 序列：颜色、光标、清屏、滚动区。
- 支持全屏程序所需 alternate screen 行为。
- 维护屏幕缓冲与光标状态。

#### 接口

- `feed(bytes)`：输入远端输出
- `render(frame)`：输出渲染帧
- `resize(cols, rows)`：更新尺寸并回传上游

#### 可测试性

- 转义序列回归用例（golden tests）。
- 全屏程序关键行为模拟（进入/退出 alternate screen）。

---

### 3.7 `tui_shell` 模块（界面与交互）

#### 职责

- 绘制 Tab 栏、会话区、状态栏/错误提示。
- 将用户键盘事件分发为：本地控制命令或远端输入。

#### 快捷键方案（TBD-1 最终设计）

为尽量避免与远端程序和系统终端冲突，采用**前缀键模式**：

- 前缀键：`Ctrl+A`
- `Ctrl+A, c`：新建 Tab（弹出连接串输入）
- `Ctrl+A, x`：关闭当前 Tab
- `Ctrl+A, n`：切换到下一个 Tab
- `Ctrl+A, p`：切换到上一个 Tab
- `Ctrl+A, Ctrl+A`：发送字面 `Ctrl+A` 给远端

说明：

- 非前缀模式下，按键默认透传给远端会话，降低对 `vim/top/less` 的干扰。
- 前缀超时（如 1 秒）后自动回退普通输入模式。

#### 可测试性

- 输入状态机可单测（前缀/透传/超时）。
- UI 绘制做快照测试（Tab 标题、激活态、错误提示）。

---

### 3.8 `error_model` 模块（错误与退出码聚合）

#### 职责

- 统一错误类型与退出码映射。
- 管理多 Tab 场景下“首个错误码”策略。
- 统一 stderr 输出格式：`[ERROR] <code>: <message>`。

#### 规则

1. 用户主动关闭全部 Tab：退出码 0。
2. 单 Tab 异常仅影响该 Tab，不立即导致进程退出。
3. 全部 Tab 异常结束时返回首个错误码。
4. 未分类错误回落到 1。

#### 可测试性

- 聚合器状态机测试（混合成功/失败关闭顺序）。

---

## 4. 事件流与关键时序

### 4.1 启动时序

1. `cli` 解析参数并生成连接列表。
2. `app core` 初始化 `session_manager` 与 `tui_shell`。
3. 为每个连接串创建会话任务（Tokio）。
4. 首帧渲染后进入统一事件循环。

### 4.2 运行时新建 Tab

1. 用户触发 `Ctrl+A, c`。
2. `tui_shell` 打开输入框，提交连接串。
3. `connection_spec` 校验通过后创建新会话。
4. `session_manager` 更新激活 Tab 并触发重绘。

### 4.3 终端尺寸变化

1. `tui_shell` 捕获窗口 resize。
2. 将新尺寸广播给激活会话或全部会话（按策略）。
3. `ssh_runtime` 调用 `resize` 同步到远端 PTY。

---

## 5. 并发与任务模型

- 基于 Tokio 多任务：
  - `UI Event Task`：输入/重绘。
  - `Session Task x N`：每个 Tab 独立收发循环。
  - `Coordinator Task`：事件汇聚与退出决策。
- 会话任务互不阻塞，单会话异常不传播至其它会话任务。
- 使用消息通道（mpsc/watch）进行模块通讯，避免共享可变状态扩散。

---

## 6. 第三方库选型（含对比）

### 6.1 SSH 库

候选：

1. **`russh`**
   - 优点：纯 Rust、异步友好、可控性高。
   - 缺点：相对生态体量较小，接入复杂度中等。
2. `ssh2`（libssh2 绑定）
   - 优点：成熟度高。
   - 缺点：依赖 C 库，跨平台单文件分发复杂度更高。

MVP 选择：**`russh`**（优先纯 Rust 与 Tokio 兼容性）。

### 6.2 TUI 库

候选：

1. **`ratatui` + `crossterm`**
   - 优点：Rust 生态主流，跨平台终端输入输出能力稳定，组件化易扩展。
   - 缺点：需自行组织较完整状态管理。
2. `cursive`
   - 优点：上手快。
   - 缺点：复杂终端仿真与多会话控制灵活性不足。

MVP 选择：**`ratatui` + `crossterm`**。

### 6.3 PTY/ConPTY 库

候选：

1. **`portable-pty`**
   - 优点：统一 Unix PTY/Windows ConPTY 抽象，降低平台分支成本。
   - 缺点：需验证高并发场景下表现。
2. 平台原生 API 直连
   - 优点：可做深度定制。
   - 缺点：开发和维护成本高，不利于 MVP 迭代速度。

MVP 选择：**`portable-pty`**。

### 6.4 异步运行时与错误处理

- 运行时：**`tokio`**（满足多会话并发）。
- 错误建模：`thiserror`（统一错误类型定义）。
- 日志（可选）：`tracing` + `tracing-subscriber`（MVP 保持最小化，可按需启用）。

---

## 7. 测试设计（按模块独立）

### 7.1 单元测试

- `cli`：参数解析与错误分支。
- `connection_spec`：格式解析与字段约束。
- `session_manager`：Tab 状态机。
- `error_model`：退出码聚合规则。
- `tui_shell`：快捷键前缀状态机。

### 7.2 集成测试

- 单会话连接成功/失败路径（网络错误、认证失败）。
- 多 Tab 并发连接与单 Tab 异常隔离。
- resize 同步与全屏程序基础行为。

### 7.3 平台验证

- Linux/macOS：PTY 功能与 ANSI 行为。
- Windows（Build 17763+）：ConPTY 启动、输入输出、窗口缩放。

---

## 8. 与 Qt 集成的接口契约

- Qt 通过 `QProcess::start("sshell", args)` 启动进程。
- Qt 不嵌入 UI，仅消费进程生命周期与退出码。
- sshell 保证退出码语义与 stderr 错误格式稳定。

---

## 9. 需求映射矩阵（MVP）

| 需求项 | 设计模块 |
|---|---|
| 建立 SSH 连接 | `cli` + `connection_spec` + `ssh_runtime` |
| 断开连接 | `session_manager` + `ssh_runtime` |
| 多 Tab 管理 | `session_manager` + `tui_shell` |
| 交互式终端（PTY/ANSI/全屏） | `pty_adapter` + `terminal_engine` + `tui_shell` |
| 标准化错误码 | `error_model` + `app core` |

---

## 10. 风险与缓解

1. **复杂终端序列兼容风险**  
   缓解：以回归样例覆盖常见全屏程序关键序列。
2. **Windows 终端差异风险**  
   缓解：ConPTY 独立适配层 + Build 版本前置检查。
3. **快捷键冲突风险**  
   缓解：采用前缀键模式并保留后续可配置化能力。

---

## 11. 里程碑对应实现建议（MVP 内）

- M1：`cli`、`connection_spec`、`ssh_runtime`、单会话终端链路打通。
- M2：`session_manager` + `tui_shell` 多 Tab 与快捷键。
- M3：`pty_adapter` Windows ConPTY 完成并验证 Build 17763+。
- M4：`error_model` 收敛、退出码稳定化、打包交付。

---

*本设计文档只覆盖 MVP，并以模块独立、可独立测试为核心约束。*

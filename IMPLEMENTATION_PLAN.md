# Codex 重置预警——实施计划

> **供执行代理使用：** 必需子技能：使用 `superpowers:subagent-driven-development`（推荐）或 `superpowers:executing-plans`，按任务逐项实施本计划。各步骤使用复选框（`- [ ]`）语法跟踪进度。

**目标：** 构建并部署一款单设备 Android Material 3 应用，由 Rust 服务提供后端支持；该服务按北京时间每小时监测三个公开来源，并通过 FCM 和 Gmail 发送去重后的 A/B 级额度重置预警。

**架构：** Android 应用采用 Kotlin/Jetpack Compose 客户端。Rust/Axum 服务负责调度、来源适配、证据规范化、确定性规则、GPT-5.6 Terra 分类、SQLite 持久化、设备配对、FCM 投递和 Gmail 投递。Nginx 使用短期 Let’s Encrypt IP 证书，在固定公网 IPv4 上终止 HTTPS，并且只把经过认证的 API 流量反向代理到本机地址。

**技术栈：** Android Kotlin `2.2.21`、Compose BOM `2025.10.00`、Material 3、Firebase Messaging BOM `34.3.0`、Rust `1.89.0` edition 2024、Axum `0.8.4`、Tokio `1.47.1`、SQLx `0.8.6`、SQLite `3.46.1`、Nginx `1.28.0`、Certbot `5.4.0`，并通过 OpenAI 兼容 API 使用 GPT 模型 `gpt-5.6-terra`。

## 全局约束

- 时区固定为 `Asia/Shanghai`；计划检查在每天 08:00 至 23:00 的每个整点运行，00:00 至 07:59 不执行计划检查。
- 08:00 的运行范围为：前一个自然日 00:00:00（含）至当前自然日 08:00:00（不含）。
- 只有 X `@thsottiaux`、`https://www.willcodexquotareset.com/` 和 `https://status.openai.com/` 三类来源具备判定资格。
- A/B 级规则、官方信号优先于预测的冲突处理规则、四行输出格式和忽略规则均复制自 `PRD.md`，不得为了实现方便而降低标准。
- 历史记录、原始证据、运行记录和通知载荷保留 30 天。
- 仅绑定一台 Android 设备；配对使用一个八字符配对码，有效期 10 分钟，且只能使用一次。
- 不实现 Gmail 登录或多用户账户系统。
- Android 包名为 `com.xingheluqi.codexresetwatch`。
- 生产环境 Android 流量仅允许通过 `https://<fixed-public-ip>/api` 使用 HTTPS；生产环境不得使用明文 HTTP。
- v1 不运行本地模型或常驻浏览器；使用有明确边界的 HTTP 抓取和已配置的模型中转服务。
- 凭据不得提交到版本库、粘贴到源代码中或输出到日志。
- 每个实施步骤都必须以有针对性的测试或验证命令结束。

## 1. 仓库与工具链初始化

### 任务 1.1：创建仓库骨架

**文件：**

- 创建：`server/`
- 创建：`android/`
- 创建：`contracts/`
- 创建：`infra/`
- 创建：`scripts/`
- 修改：`.gitignore`

**步骤：**

- [ ] 创建上述目录，并创建根目录 `README.md`，其中链接到六份规范文件。
- [ ] 在 `.gitignore` 中加入 `.env`、`*.keystore`、Firebase 服务账号 JSON、`server/target/`、`android/.gradle/`、`android/local.properties`、APK 输出文件、Nginx 私钥以及 SQLite 运行时文件。
- [ ] 验证 `git status --short` 只显示预期的仓库骨架和文档文件。

**验证：**

```bash
find server android contracts infra scripts -maxdepth 1 -type d -print
git status --short
```

### 任务 1.2：锁定 Rust 工具链与工作区

**文件：**

- 创建：`server/rust-toolchain.toml`
- 创建：`server/Cargo.toml`
- 创建：`server/src/main.rs`

**步骤：**

- [ ] 在服务端工具链文件中设置 `channel = "1.89.0"`、`profile = "minimal"` 和目标平台 `aarch64-unknown-linux-gnu`。
- [ ] 在 `server/Cargo.toml` 中设置 `edition = "2024"` 和包版本 `1.0.0`。
- [ ] 加入 `TECH_STACK.md` 中规定的精确依赖版本；只启用其中列出的功能标志。
- [ ] 让 `main.rs` 启动 Tokio 运行时并返回 `Ok(())`。
- [ ] 生成并提交 `server/Cargo.lock`。

**验证：**

```bash
cd server
rustup show active-toolchain
cargo check --locked
cargo test --locked
```

### 任务 1.3：锁定 Android 工具链

**文件：**

- 创建：`android/settings.gradle.kts`
- 创建：`android/build.gradle.kts`
- 创建：`android/gradle/libs.versions.toml`
- 创建：`android/gradle.properties`
- 创建：`android/app/build.gradle.kts`
- 创建：`android/app/src/main/AndroidManifest.xml`

**步骤：**

- [ ] 配置 Gradle `8.13`、AGP `8.13.0`、Kotlin `2.2.21`、编译/目标 SDK `36` 和最低 SDK `28`。
- [ ] 将命名空间和应用 ID 设置为 `com.xingheluqi.codexresetwatch`。
- [ ] 通过版本目录加入 `TECH_STACK.md` 中规定的精确 Android 依赖。
- [ ] 按锁定版本启用 Compose 和 Kotlin serialization 插件。
- [ ] 加入 `android:networkSecurityConfig`，并将生产环境明文流量设置为禁用。
- [ ] Firebase 配置只能通过本地 `google-services.json` 路径加入；该凭据文件必须排除在版本控制之外。

**验证：**

```bash
cd android
./gradlew --version
./gradlew :app:assembleDebug
./gradlew :app:testDebugUnitTest
```

## 2. 后端基础

### 任务 2.1：定义配置与类型化错误

**文件：**

- 创建：`server/src/config.rs`
- 创建：`server/src/error.rs`
- 修改：`server/src/main.rs`
- 创建：`server/.env.example`

**步骤：**

- [x] 为数据库 URL、绑定地址、固定公网 IP、模型基础 URL、模型 API 密钥、模型名称、Gmail 凭据、FCM 项目 ID、FCM 服务账号路径、保留天数和配对限制定义类型化配置字段。
- [x] 当缺少生产环境必需密钥或 `MODEL_NAME != gpt-5.6-terra` 时拒绝启动。
- [x] 定义与 `BACKEND_STRUCTURE.md` 一致的公开错误码。
- [x] 在 `Debug` 实现和日志中遮盖密钥。
- [x] 创建 `.env.example`，只包含变量名和安全的虚拟值。

**测试：**

- [x] 测试缺少生产环境密钥时配置加载失败。
- [x] 测试 `Asia/Shanghai`、08:00、23:00 和 30 天均为默认值。
- [x] 测试错误序列化结果包含 `error.code`、`message`、`retryable` 和 `request_id`。

**验证：**

```bash
cd server
cargo test --locked config
cargo test --locked error
```

### 任务 2.2：创建数据库迁移与模型

**文件：**

- 创建：`server/migrations/0001_initial.sql`
- 创建：`server/src/db/mod.rs`
- 创建：`server/src/db/models.rs`
- 创建：`server/src/db/queries.rs`
- 修改：`server/src/main.rs`

**步骤：**

- [x] 实现 `BACKEND_STRUCTURE.md` 中的每一张表及每一项约束。
- [x] 在每个连接上启用 SQLite 外键和 WAL 模式。
- [x] 加入所有必需的唯一约束和索引。
- [x] 为来源条目、预警、证据、运行记录、通知、设备、配对码和偏好设置实现类型化 Rust 模型。
- [x] 加入来源游标事务、预警更新插入、证据插入、通知幂等和保留期清理查询。

**测试：**

- [x] 对临时 SQLite 数据库应用迁移。
- [x] 将同一个来源条目插入两次，并验证唯一约束生效。
- [x] 插入带证据的预警，并验证外键关系。
- [x] 将相同的通知去重键插入两次，并验证只接受一条逻辑记录。

**验证：**

```bash
cd server
cargo test --locked db
sqlx database create --database-url sqlite://test.db
sqlx migrate run --source migrations --database-url sqlite://test.db
```

### 任务 2.3：实现北京时间调度

**文件：**

- 创建：`server/src/time.rs`
- 创建：`server/src/scheduler/hourly.rs`
- 创建：`server/src/scheduler/manual.rs`
- 创建：`server/src/scheduler/mod.rs`

**步骤：**

- [x] 使用 `chrono-tz` 和 `Asia/Shanghai` 实现 `next_scheduled_run(now_utc) -> Option<DateTime<Utc>>`。
- [x] 北京本地时间 00:00 至 07:59 之间不返回计划运行。
- [x] 08:00 至 22:59 返回下一个整点运行，并将 23:00 包含为当天最后一次运行。
- [x] 将 08:00 补查窗口生成为：前一个本地自然日 00:00:00（含）至当前本地自然日 08:00:00（不含）。
- [x] 实现手动运行受理机制，设置五分钟频率限制，并复用正在执行的运行。

**测试：**

- [x] 测试 07:59:59 返回 08:00 运行。
- [x] 测试当前 08:00 运行被受理后，08:00 返回下一次 09:00 运行。
- [x] 测试 23:00 不返回当天更晚的运行，并返回次日 08:00 运行。
- [x] 测试跨月和跨年时 08:00 补查窗口边界。
- [x] 测试手动检查频率限制和重复运行复用。

**验证：**

```bash
cd server
cargo test --locked scheduler
cargo test --locked time
```

## 3. 来源适配器与证据规范化

### 任务 3.1：构建有边界的 HTTP 客户端

**文件：**

- 创建：`server/src/sources/http.rs`
- 修改：`server/src/config.rs`

**步骤：**

- [ ] 使用 rustls 配置 `reqwest`，重定向上限为 5，连接超时为 10 秒，读取超时为 20 秒，总超时为 30 秒，并设置清晰的用户代理。
- [ ] 除本地测试模拟服务外，拒绝非 HTTPS 来源 URL。
- [ ] 捕获状态码、最终 URL、ETag、Last-Modified、内容类型、响应长度和经过清理的错误信息。
- [ ] 将 HTML 来源的响应正文限制为 2 MiB。
- [ ] 不记录响应正文或授权请求头。

**测试：**

- [ ] 模拟额度预测根域名到 `www` URL 的 307 重定向，并验证最终 URL 被接受。
- [ ] 模拟超时、DNS 失败以及 403、404、500 响应。
- [ ] 验证超过大小上限的响应正文会被作为来源错误拒绝。

### 任务 3.2：实现 X `@thsottiaux` 适配器

**文件：**

- 创建：`server/src/sources/x_thsottiaux.rs`
- 创建：`server/src/sources/normalize.rs`

**步骤：**

- [ ] 在规范 URL 归一化后，将可接受的作者身份严格限制为 `thsottiaux`。
- [ ] 只解析无需私有账号会话即可获取的公开帖子和公开回复。
- [ ] 提取帖子 ID、作者、发布时间、帖子正文、回复父项、线程根项、规范 URL 和公开上下文。
- [ ] 规范化空白、Unicode 标点、URL 跟踪参数和不区分大小写的关键词匹配，同时不得破坏引用文本。
- [ ] 将 X 页面结构变化视为来源失败，而不是空来源。

**测试：**

- [ ] 解析一条包含明确未来 usage-limit 承诺的帖子。
- [ ] 解析一条由父项补全缺失 Codex 额度对象的直接回复。
- [ ] 拒绝其他作者以及私有或不可用内容。
- [ ] 验证相同内容会生成相同的内容哈希。

### 任务 3.3：实现额度预测适配器

**文件：**

- 创建：`server/src/sources/quota_forecast.rs`

**步骤：**

- [ ] 使用 `https://www.willcodexquotareset.com/` 作为规范 URL。
- [ ] 解析概率、预测窗口、页面显示时区、页面更新时间，以及页面稳定观测标识符（如可用）。
- [ ] 将概率转换为整数基点。
- [ ] 在证据中保留页面显示的原始值，同时存储规范化数值用于阈值比较。
- [ ] 将概率缺失或窗口格式错误视为来源降级，不得视为零概率。

**测试：**

- [ ] 解析 70%、82% 和 100% 的值。
- [ ] 验证 69.99% 不满足 70% 阈值。
- [ ] 验证根域名重定向会被规范化为 `www` 规范 URL。
- [ ] 验证页面布局解析失败不会产生空的成功观测结果。

### 任务 3.4：实现 OpenAI Status Codex 适配器

**文件：**

- 创建：`server/src/sources/openai_status.rs`

**步骤：**

- [ ] 获取公开状态页面及其公开事件历史/更新数据。
- [ ] 筛选与 Codex 相关的组件或事件。
- [ ] 提取事件 ID、标题、状态、更新文本、发布时间、解决时间和规范 URL。
- [ ] 将仅描述已结束事件的内容标记为历史证据；若不存在未来补偿或恢复信号，不得据此生成未来预警。

**测试：**

- [ ] 解析一条提及补偿的活动中 Codex 容量事件。
- [ ] 解析一条已解决事件，并验证该事件本身不会触发未来预警。
- [ ] 拒绝与 Codex 无关、仅涉及 ChatGPT 的事件。

## 4. 分类、聚类与通知决策

### 任务 4.1：实现确定性预筛选

**文件：**

- 创建：`server/src/classification/rules.rs`
- 创建：`server/src/classification/clustering.rs`

**步骤：**

- [ ] 为重置、usage limits、rate limits、quota、capacity、restore、compensation 和未来时间表达式定义规范化关键词族。
- [ ] 拒绝普通产品或模型发布文本，除非其中同时包含符合要求的额度对象和未来动作。
- [ ] 拒绝仅描述已完成动作的语言，例如 “has been reset”，除非还存在后续未来动作。
- [ ] A 级候选必须包含官方来源。
- [ ] 使用规范化额度对象、事件窗口分桶、官方来源身份和直接线程根项创建稳定的 `cluster_key`。

**测试：**

- [ ] 为 `PRD.md` 中的每一条忽略规则加入表驱动测试。
- [ ] 加入测试，证明没有官方信号支持的预测概率会被忽略。
- [ ] 加入测试，证明直接回复上下文可以补全原本不完整的额度对象。
- [ ] 加入属性测试，验证空白和标点规范化后聚类键保持稳定。

### 任务 4.2：加入 OpenAI 兼容模型分类器

**文件：**

- 创建：`server/src/classification/prompt.rs`
- 创建：`server/src/classification/model_client.rs`
- 创建：`server/src/classification/schema.rs`

**步骤：**

- [ ] 创建版本化提示词 `classifier-v1`，把所有来源文本视为不可信数据。
- [ ] 只把经过预筛选的候选项及其直接上下文发送给 `gpt-5.6-terra`。
- [ ] 使用 temperature 0 请求确定性 JSON，并限制输出长度。
- [ ] 严格解析 `BACKEND_STRUCTURE.md` 中定义的模式。
- [ ] 拒绝未知决策值、缺失的证据句、无效时间窗口和额外说明文字。
- [ ] 不存储原始提示词或提供商密钥；只存储响应哈希和经过验证的判定依据。

**测试：**

- [ ] 模拟一个有效的 A 级结果。
- [ ] 模拟一个有效的 B 级结果。
- [ ] 为已完成的重置模拟一个 `none` 结果。
- [ ] 模拟格式错误的 JSON、额外 Markdown、超时以及 429、500 响应。
- [ ] 验证无效分类结果不会进入通知创建流程。

### 任务 4.3：实现确定性 A/B 后置校验

**文件：**

- 修改：`server/src/classification/rules.rs`
- 修改：`server/src/classification/schema.rs`

**步骤：**

- [ ] 独立于模型输出强制执行 A 级要求。
- [ ] 独立于模型输出强制执行 B1 的官方信号要求。
- [ ] 强制执行官方信号优先于预测的冲突处理规则。
- [ ] 将时间窗口换算为北京时间，用于面向用户的摘要。
- [ ] 生成严格的四行摘要和建议。

**测试：**

- [ ] 验证缺少额度对象时，模型给出的 A 级结果会被降为 `none`。
- [ ] 验证预测概率较低时，官方未来承诺仍然保留为预警。
- [ ] 验证缺少官方信号的 B 级预测会变为 `none`。
- [ ] 验证已完成的重置会变为 `none`。

### 任务 4.4：实现预警聚类与去重

**文件：**

- 创建：`server/src/notifications/dedup.rs`
- 修改：`server/src/classification/clustering.rs`
- 修改：`server/src/db/queries.rs`

**步骤：**

- [ ] 每个逻辑重置事件创建或更新一条 `signal_clusters` 记录。
- [ ] 存储初始等级、当前等级、窗口、证据摘要、冲突状态和建议。
- [ ] 对首个符合条件的预警发出事件类型 `initial`。
- [ ] 只有同一聚类从 B 变为 A 时才发出 `upgrade`。
- [ ] 只有发生 `PRD.md` 明确定义的实质变化时才发出 `material_update`。
- [ ] 抑制未变化内容和重复的预测快照。

**测试：**

- [ ] 将同一个来源条目处理两次，并验证不会新增通知。
- [ ] 依次处理 B、A，并验证只产生一个升级事件。
- [ ] 依次处理 A 和更低的预测概率，并验证 A 级预警仍保持活动状态。
- [ ] 处理新的官方上下文，并验证只产生一次实质更新。

### 任务 4.5：实现 FCM 和 Gmail 投递

**文件：**

- 创建：`server/src/notifications/fcm.rs`
- 创建：`server/src/notifications/email.rs`
- 修改：`server/src/notifications/mod.rs`

**步骤：**

- [ ] 实现 FCM HTTP v1 OAuth 服务账号断言和单令牌发送。
- [ ] 在 `smtp.gmail.com:465` 上实现基于 TLS 的 SMTP。
- [ ] 从共享预警对象构建 A/B 级邮件主题和完整正文。
- [ ] 发送前使用幂等键，并在发送后记录提供商结果。
- [ ] 对临时 FCM/SMTP 错误使用有上限的指数退避重试；绝不创建第二条逻辑预警。
- [ ] 遵守 `user_preferences.push_enabled` 和 `email_enabled` 设置。

**测试：**

- [ ] 模拟 FCM 和 SMTP 投递成功。
- [ ] 模拟提供商临时失败，并验证重试状态。
- [ ] 模拟提供商永久失败，并验证失败状态且不生成重复预警。
- [ ] 验证邮件内容包含四行摘要、来源链接、北京时间、证据和冲突文本。
- [ ] 验证被禁用的渠道会被标记为已抑制，而不是失败。

## 5. API 与部署安全

### 任务 5.1：实现配对与身份认证

**文件：**

- 创建：`server/src/auth/pairing_codes.rs`
- 创建：`server/src/auth/device_tokens.rs`
- 创建：`server/src/auth/middleware.rs`
- 创建：`server/src/api/auth.rs`

**步骤：**

- [ ] 使用允许的字符表和 Argon2id 哈希实现 CLI 配对码生成。
- [ ] 实现一次性领取、十分钟过期、五次尝试锁定和单活动设备规则。
- [ ] 签发随机不透明令牌，并且只存储其哈希。
- [ ] 为受保护端点加入 Bearer 中间件。
- [ ] 实现撤销和心跳，并对 FCM 令牌加密。
- [ ] 从日志中遮盖所有配对码、身份令牌和 FCM 令牌。

**测试：**

- [ ] 领取一次有效配对码，并验证第二次领取失败。
- [ ] 验证过期、锁定、格式错误和错误配对码。
- [ ] 验证已撤销的 Bearer 令牌返回 401。
- [ ] 验证第一台设备处于活动状态时，第二台设备会被拒绝。

### 任务 5.2：实现 REST 端点

**文件：**

- 创建：`server/src/api/mod.rs`
- 创建：`server/src/api/health.rs`
- 创建：`server/src/api/pairing.rs`
- 创建：`server/src/api/home.rs`
- 创建：`server/src/api/alerts.rs`
- 创建：`server/src/api/runs.rs`
- 创建：`server/src/api/preferences.rs`
- 创建：`contracts/openapi.yaml`

**步骤：**

- [ ] 实现 `BACKEND_STRUCTURE.md` 中规定的精确端点和响应封装。
- [ ] 加入请求 ID 中间件和一致的错误响应。
- [ ] 为预警和运行记录加入游标分页。
- [ ] 返回带明确时区偏移的北京时间显示值。
- [ ] 为 `POST /api/v1/checks` 加入队列语义和五分钟频率限制。
- [ ] 根据已批准的端点合约生成 `contracts/openapi.yaml` 并进行验证。

**测试：**

- [ ] 使用有效认证、缺失认证、无效认证、过期数据和服务端错误状态测试每个端点。
- [ ] 验证 `GET /healthz` 保持公开，而 `GET /api/v1/home` 要求认证。
- [ ] 验证分页游标和 30 天范围限制。
- [ ] 对 `contracts/openapi.yaml` 运行 OpenAPI 检查工具。

### 任务 5.3：实现保留期与运行健康检查

**文件：**

- 创建：`server/src/retention.rs`
- 修改：`server/src/scheduler/mod.rs`
- 创建：`server/src/bin/admin.rs`

**步骤：**

- [ ] 每天北京时间 03:30 运行保留期清理。
- [ ] 删除所有超过 30 天的指定记录，同时保留当前聚类引用，直至可以安全删除。
- [ ] 为迁移、配对码生成、设备撤销和健康报告加入管理员命令。
- [ ] 加入结构化 JSON 日志，其中包含来源、运行 ID、预警 ID、耗时和经过遮盖的错误。

**测试：**

- [ ] 插入距今 29、30、31 天的记录，并验证只删除超过保留边界的记录。
- [ ] 验证包含近期证据的保留聚类不会被删除。
- [ ] 验证管理员 CLI 绝不输出敏感配置值。

### 任务 5.4：将服务容器化

**文件：**

- 创建：`infra/Dockerfile`
- 创建：`infra/docker-compose.yml`
- 创建：`infra/server.env.example`
- 创建：`infra/systemd/codex-reset-watch.service`

**步骤：**

- [ ] 使用精确锁定的工具链构建多阶段 Rust 镜像。
- [ ] 在容器内以非 root 用户运行。
- [ ] 挂载 SQLite 数据，并在适用时以只读方式挂载 FCM 服务账号 JSON。
- [ ] 设置适用于 2 vCPU / 4 GiB RAM 的内存和 CPU 限制。
- [ ] 配置重启策略，并针对 `/healthz` 配置健康检查。
- [ ] 让 Rust 在主机内部只绑定到 `127.0.0.1:8080`。

**验证：**

```bash
docker compose -f infra/docker-compose.yml config
docker compose -f infra/docker-compose.yml build --pull
docker compose -f infra/docker-compose.yml up -d
curl -fsS http://127.0.0.1:8080/healthz
```

### 任务 5.5：配置 Nginx 和 IP HTTPS

**文件：**

- 创建：`infra/nginx.conf`
- 创建：`infra/certbot/renew-ip-cert.sh`
- 创建：`infra/systemd/codex-reset-watch-renew.timer`

**步骤：**

- [ ] 80 端口只用于 ACME 质询和 HTTPS 重定向。
- [ ] 使用固定 IPv4 证书和现代 TLS 设置监听 443 端口。
- [ ] 将 `/api/` 反向代理到 `127.0.0.1:8080`。
- [ ] 加入请求正文大小和超时限制。
- [ ] 在签发生产证书前，使用该 IP 地址和短期证书配置文件测试 Certbot 暂存环境。
- [ ] 每四天自动续期，成功续期后重新加载 Nginx，并在失败时提醒运维人员。
- [ ] 服务器防火墙只开放 22/SSH、80 和 443；SQLite 与 Rust 端口保持私有。

**验证：**

```bash
sudo nginx -t
curl -I http://<fixed-public-ip>/healthz
curl -fsS https://<fixed-public-ip>/healthz
sudo systemctl list-timers codex-reset-watch-renew.timer
```

## 6. Android 客户端实施

### 任务 6.1：构建 Material 3 主题与应用外壳

**文件：**

- 创建：`android/app/src/main/java/com/xingheluqi/codexresetwatch/ui/theme/Color.kt`
- 创建：`android/app/src/main/java/com/xingheluqi/codexresetwatch/ui/theme/Theme.kt`
- 创建：`android/app/src/main/java/com/xingheluqi/codexresetwatch/ui/theme/Type.kt`
- 创建：`android/app/src/main/java/com/xingheluqi/codexresetwatch/ui/App.kt`
- 创建：`android/app/src/main/java/com/xingheluqi/codexresetwatch/ui/navigation/AppNavHost.kt`

**步骤：**

- [ ] 实现 `FRONTEND_GUIDELINES.md` 中定义的深灰黑色角色和 A/B/正常语义色。
- [ ] 配置 Roboto，并使用简体中文系统字体作为回退字体。
- [ ] 实现 `Scaffold`、顶部应用栏、底部 `NavigationBar` 和窗口边衬区处理。
- [ ] 为配对、主页、证据、预警详情、运行记录、运行详情、设置和偏好设置定义路由。
- [ ] 为正常、降级、B 级和 A 级状态加入预览固定数据。

**验证：**

```bash
cd android
./gradlew :app:assembleDebug
./gradlew :app:testDebugUnitTest
```

### 任务 6.2：实现 API 客户端与本地安全状态

**文件：**

- 创建：`android/app/src/main/java/com/xingheluqi/codexresetwatch/data/ApiClient.kt`
- 创建：`android/app/src/main/java/com/xingheluqi/codexresetwatch/data/ApiModels.kt`
- 创建：`android/app/src/main/java/com/xingheluqi/codexresetwatch/data/DeviceCredentialStore.kt`
- 创建：`android/app/src/main/res/xml/network_security_config.xml`

**步骤：**

- [ ] 实现 Ktor HTTPS 客户端，支持请求 ID、客户端版本请求头、Bearer 认证、JSON 序列化和有明确边界的超时。
- [ ] 通过基于 Android Keystore 的加密和 DataStore 存储设备令牌。
- [ ] 实现缓存的主页、预警、运行记录和偏好设置模型。
- [ ] 将 HTTP 错误映射为类型化 UI 错误，不得泄露服务端堆栈跟踪。
- [ ] 通过发布构建配置值设置固定 IPv4 基础 URL，不得在源代码中重复字符串。

**测试：**

- [ ] 对每个 API 响应封装固定数据执行序列化和反序列化。
- [ ] 验证 `401` 只会在明确重新配对流程后清除会话状态。
- [ ] 验证离线缓存无需网络请求即可渲染。
- [ ] 验证发布配置会拒绝明文 HTTP 请求。

### 任务 6.3：实现配对流程

**文件：**

- 创建：`android/app/src/main/java/com/xingheluqi/codexresetwatch/feature/pairing/PairingScreen.kt`
- 创建：`android/app/src/main/java/com/xingheluqi/codexresetwatch/feature/pairing/PairingViewModel.kt`
- 创建：`android/app/src/main/java/com/xingheluqi/codexresetwatch/data/PairingRepository.kt`

**步骤：**

- [ ] 在本地验证八位大写字母和数字字符。
- [ ] 在领取配对前获取 FCM 令牌。
- [ ] 提交配对元数据，并且只在收到 `201` 响应后存储返回的令牌。
- [ ] 实现 `APP_FLOW.md` 中的所有配对错误状态。
- [ ] 领取成功后导航到主页。

**测试：**

- [ ] 测试本地校验，并验证格式错误的配对码不会发起请求。
- [ ] 测试成功、过期、已使用、已锁定、已绑定和网络失败状态。
- [ ] 测试不完整的服务端响应绝不会导致凭据被存储。

### 任务 6.4：实现主页与运行状态

**文件：**

- 创建：`android/app/src/main/java/com/xingheluqi/codexresetwatch/feature/home/HomeScreen.kt`
- 创建：`android/app/src/main/java/com/xingheluqi/codexresetwatch/feature/home/HomeViewModel.kt`
- 创建：`android/app/src/main/java/com/xingheluqi/codexresetwatch/ui/components/RuntimeStatusArea.kt`
- 创建：`android/app/src/main/java/com/xingheluqi/codexresetwatch/ui/components/StatusCard.kt`
- 创建：`android/app/src/main/java/com/xingheluqi/codexresetwatch/ui/components/SourceHealthChips.kt`

**步骤：**

- [ ] 渲染唯一的运行状态区域，其中包含服务状态、上次检查、下次检查、北京时间标识和“立即检查”。
- [ ] 渲染正常、降级、B 级和 A 级卡片。
- [ ] 保持主页紧凑，避免在卡片中重复运行数据。
- [ ] 实现缓存优先加载和刷新。
- [ ] 实现手动检查受理、执行中、冷却和超时状态。

**测试：**

- [ ] 在 320dp、360dp 和 412dp 宽度下对正常与预警布局进行 Compose 测试。
- [ ] 验证无需水平滚动。
- [ ] 验证使用大号字体比例时按钮仍可触达。
- [ ] 验证界面只渲染一个“立即检查”控件。

### 任务 6.5：实现证据、运行记录与设置

**文件：**

- 创建：`android/app/src/main/java/com/xingheluqi/codexresetwatch/feature/evidence/EvidenceScreen.kt`
- 创建：`android/app/src/main/java/com/xingheluqi/codexresetwatch/feature/evidence/AlertDetailScreen.kt`
- 创建：`android/app/src/main/java/com/xingheluqi/codexresetwatch/feature/runs/RunsScreen.kt`
- 创建：`android/app/src/main/java/com/xingheluqi/codexresetwatch/feature/runs/RunDetailScreen.kt`
- 创建：`android/app/src/main/java/com/xingheluqi/codexresetwatch/feature/settings/SettingsScreen.kt`

**步骤：**

- [ ] 实现证据筛选、游标分页、预警详情、引用展开和外部来源链接。
- [ ] 实现运行记录列表和运行详情，并区分“无预警”与“部分失败”。
- [ ] 实现推送/邮件偏好开关和测试通知操作。
- [ ] 实现重新配对确认和服务端撤销流程。
- [ ] 从浏览器返回后保留缓存数据和滚动位置。

**测试：**

- [ ] 测试空、已有数据、已归档和已过期的证据状态。
- [ ] 测试来源冲突卡片渲染。
- [ ] 测试全部四种结果状态的运行状态颜色和文字。
- [ ] 测试偏好设置更新失败时恢复开关之前的状态。
- [ ] 测试取消重新配对时保留活动凭据。

### 任务 6.6：实现 FCM 通知处理

**文件：**

- 创建：`android/app/src/main/java/com/xingheluqi/codexresetwatch/notifications/CodexFirebaseMessagingService.kt`
- 创建：`android/app/src/main/java/com/xingheluqi/codexresetwatch/notifications/NotificationChannels.kt`
- 修改：`android/app/src/main/AndroidManifest.xml`
- 修改：`android/app/src/main/java/com/xingheluqi/codexresetwatch/data/ApiClient.kt`

**步骤：**

- [ ] 创建 A 级高重要性通知渠道和 B 级默认重要性通知渠道。
- [ ] 通过调用 `/api/v1/device/heartbeat` 处理 FCM 令牌创建和刷新。
- [ ] 根据四行摘要字段渲染通知标题和正文。
- [ ] 将 `alert_id` 和 `event_type` 放入 intent extras。
- [ ] 将通知点击操作深层链接到预警详情页。
- [ ] 在设置页显示通知权限被禁用的诊断信息。

**测试：**

- [ ] 使用 Firebase 测试消息测试应用处于前台、后台和正常终止状态时的通知行为。
- [ ] 使用有效和已过期的预警 ID 测试通知点击路由。
- [ ] 测试无效 FCM 令牌的刷新路径。

## 7. 集成与验收

### 任务 7.1：加入端到端模拟环境

**文件：**

- 创建：`scripts/run-mock-stack.sh`
- 创建：`server/tests/fixtures/`
- 创建：`android/app/src/androidTest/`

**步骤：**

- [ ] 通过 WireMock 固定数据运行模拟 X、额度预测、Status、模型中转、FCM 和 SMTP 端点。
- [ ] 执行一次 08:00 补查场景，其中包含一个新的 B 级信号和一个应忽略的已完成重置信号。
- [ ] 执行一次官方信号与预测冲突的场景。
- [ ] 执行 B→A 升级，并验证每个渠道分别产生一个初始事件和一个升级事件。
- [ ] 启动连接模拟 API 的 Android 应用，并完成配对、主页、预警详情、运行记录和设置流程。

**验证：**

```bash
./scripts/run-mock-stack.sh
cd server && cargo test --locked --test end_to_end
cd ../android && ./gradlew connectedDebugAndroidTest
```

### 任务 7.2：验证香港服务器的来源连通性

**步骤：**

- [ ] 验证 `https://x.com/thsottiaux`。
- [ ] 验证 `https://www.willcodexquotareset.com/` 及其重定向行为。
- [ ] 验证 `https://status.openai.com/`。
- [ ] 在不存储密钥的前提下记录状态、最终 URL、响应时间和解析结果。
- [ ] 在启用通知前，于一天中的多个时段运行检查。

**验证：**

```bash
curl -IL --max-time 15 https://x.com/thsottiaux
curl -IL --max-time 15 https://www.willcodexquotareset.com/
curl -IL --max-time 15 https://status.openai.com/
```

### 任务 7.3：部署签名 APK 并执行真机验收测试

**步骤：**

- [ ] 在仓库外生成发布签名密钥，并安全备份。
- [ ] 使用 `com.xingheluqi.codexresetwatch` 构建签名发布版 APK。
- [ ] 将其安装到一加 13。
- [ ] 启用通知权限，在设备允许时关闭该应用的电池优化，并确认 Google Play Services 处于活动状态。
- [ ] 使用服务器生成的配对码完成配对。
- [ ] 发送一条测试 FCM 通知和一封测试邮件。
- [ ] 通过模拟分类器模拟一次预警，并验证主页、系统通知、邮件和预警详情均引用同一个预警 ID。
- [ ] 强制停止应用，记录 Android 的投递限制；重新打开应用，并验证令牌恢复。

### 任务 7.4：启用生产监测

**步骤：**

- [ ] 使用以只读方式挂载的生产密钥启动 Docker Compose 栈。
- [ ] 确认 HTTPS 和证书续期计时器。
- [ ] 确认调度器已作为 systemd 服务启用，并可在重启后继续运行。
- [ ] 确认 30 天保留期任务处于活动状态。
- [ ] 确认日志会遮盖所有密钥类别。
- [ ] 确认正常运行或来源失败时不会发送通知。
- [ ] 只有在人工审核前三次生产运行后才启用服务。

## 8. 完成定义

- [ ] 六份规范文件全部存在且内部一致。
- [ ] Rust 服务通过单元测试、解析器测试、分类器测试、API 测试、保留期测试和端到端测试。
- [ ] Android 应用在一加 13 上通过单元测试、Compose 测试和连接设备测试。
- [ ] 已从香港服务器验证三个来源。
- [ ] A/B 固定测试数据覆盖 `PRD.md` 中的每一条规则和忽略条件。
- [ ] FCM 和 Gmail 测试投递成功。
- [ ] 配对只能完成一次，且第二台设备会被拒绝。
- [ ] 无域名情况下，HTTPS 可在固定 IPv4 上正常使用。
- [ ] 调度器严格按照北京时间边界运行。
- [ ] 未提交任何密钥或公开凭据。
- [ ] 生产服务会自动启动，并且只在审核后启用。

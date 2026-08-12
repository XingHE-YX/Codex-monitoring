# Codex 重置预警——技术栈

**版本锁定日期：** 2026-08-12

**版本策略：** 所有直接依赖均锁定为下文所列的精确版本。生成的锁文件必须提交到版本库。禁止使用通配符、脱字符、波浪号、动态 Maven 版本、浮动 Docker 标签或未经审核的依赖升级。

## 1. 系统架构

```text
Android Kotlin/Compose 应用
        │ 通过固定 IPv4 使用 HTTPS
        ▼
Nginx TLS 反向代理
        │ 仅监听 localhost
        ▼
Rust Axum API + 调度器 + 信息源适配器 + 分类器 + 通知模块
        │
        ├── SQLite 数据库
        ├── 兼容 OpenAI 的模型中转服务：gpt-5.6-terra
        ├── Gmail SMTP（TLS 465）
        └── Firebase Cloud Messaging HTTP v1
```

Android 客户端绝不直接调用 X、额度预测网站、OpenAI Status、Gmail 或模型中转服务。只有 Rust 后端可以访问外部信息源和凭据。

## 2. Android 工具链

| 工具 | 精确版本 | 锁定位置 |
|---|---:|---|
| Android Studio | Ladybug Feature Drop `2024.2.2` | 开发者工作站基线 |
| JDK | `21.0.8` | `gradle.properties` / CI 镜像 |
| Gradle Wrapper | `8.13` | `android/gradle/wrapper/gradle-wrapper.properties` |
| Android Gradle Plugin | `8.13.0` | `android/settings.gradle.kts` |
| Kotlin | `2.2.21` | `android/gradle/libs.versions.toml` |
| Kotlin Compose 编译器插件 | `2.2.21` | `android/gradle/libs.versions.toml` |
| compileSdk | `36` | `android/app/build.gradle.kts` |
| targetSdk | `36` | `android/app/build.gradle.kts` |
| minSdk | `28` | `android/app/build.gradle.kts` |
| Android NDK | `27.2.12479018` | `android/app/build.gradle.kts` |

`minSdk 28` 是 v1 的最低兼容版本。主要设备为 OnePlus 13，但应用的核心流程不得依赖 Android 15 专属 API。

## 3. Android 直接依赖

Compose BOM 用于锁定彼此兼容的 Compose 构件。Compose UI 构件版本以所列的精确 BOM 版本为唯一依据。

| 组 | 构件 | 精确版本 |
|---|---|---:|
| `androidx.core` | `core-ktx` | `1.16.0` |
| `androidx.activity` | `activity-compose` | `1.10.1` |
| `androidx.lifecycle` | `lifecycle-runtime-compose` | `2.9.3` |
| `androidx.lifecycle` | `lifecycle-viewmodel-compose` | `2.9.3` |
| `androidx.navigation` | `navigation-compose` | `2.9.3` |
| `androidx.compose` | Compose BOM | `2025.10.00` |
| `androidx.compose.material3` | `material3` | 由 BOM `2025.10.00` 管理 |
| `androidx.compose.material3` | `material3-window-size-class` | 由 BOM `2025.10.00` 管理 |
| `androidx.compose.material` | `material-icons-extended` | 由 BOM `2025.10.00` 管理 |
| `androidx.datastore` | `datastore-preferences` | `1.1.7` |
| `org.jetbrains.kotlinx` | `kotlinx-coroutines-android` | `1.10.2` |
| `org.jetbrains.kotlinx` | `kotlinx-serialization-json` | `1.9.0` |
| `io.ktor` | `ktor-client-android` | `3.2.3` |
| `io.ktor` | `ktor-client-content-negotiation` | `3.2.3` |
| `io.ktor` | `ktor-serialization-kotlinx-json` | `3.2.3` |
| `io.ktor` | `ktor-client-logging` | `3.2.3` |
| `com.google.firebase` | Firebase BOM | `34.3.0` |
| `com.google.firebase` | `firebase-messaging` | 由 Firebase BOM `34.3.0` 管理 |
| `androidx.test` | `runner` | `1.7.0` |
| `androidx.test.ext` | `junit` | `1.2.1` |
| `androidx.compose.ui` | `ui-test-junit4` | 由 Compose BOM `2025.10.00` 管理 |
| `junit` | `junit` | `4.13.2` |

Android 端不使用任何身份认证 SDK，也不使用 Gmail SDK 进行用户登录。FCM 仅用于发送通知。

## 4. Android UI 与安全方案

- UI：Kotlin + Jetpack Compose + Material 3。
- 状态管理：页面级 ViewModel，使用 Kotlin `StateFlow`。
- 导航：`androidx.navigation:navigation-compose`。
- 网络通信：使用支持 JSON 序列化和 TLS 的 Ktor 客户端。
- 本地存储：非敏感设置使用 DataStore Preferences；不透明设备令牌和配对元数据使用 Android Keystore 支持的加密机制。
- 通知处理：`FirebaseMessagingService` 与通知渠道。
- 手机端不执行后台额度轮询；以服务端调度为唯一权威。
- 生产环境通过 `network_security_config.xml` 禁用明文流量。
- API 基础 URL：`https://<fixed-public-ip>/api`。
- v1 不对服务器证书实施固定，因为 IP 证书有效期较短；系统使用受信任的 TLS，并支持证书轮换。

## 5. Rust 工具链

| 工具 | 精确版本 | 锁定位置 |
|---|---:|---|
| Rust 工具链通道 | `1.89.0` | `server/rust-toolchain.toml` |
| Rust 语言版本 | `2024` | `server/Cargo.toml` |
| Cargo 锁文件格式 | 由 Rust `1.89.0` 生成 | `server/Cargo.lock` |
| Ubuntu 运行时 | `24.04.3 LTS` | 服务器镜像 |
| 容器基础镜像 | 使用已提交 SHA-256 摘要的 `debian:bookworm-slim` | `infra/Dockerfile` |

首次生产部署前，必须解析并提交容器镜像摘要。Dockerfile 必须引用解析后的不可变摘要，不得引用浮动标签。

## 6. Rust 直接依赖

| Rust 包（Crate） | 精确版本 | 用途 |
|---|---:|---|
| `axum` | `0.8.4` | HTTP API 路由 |
| `tokio` | `1.47.1` | 异步运行时 |
| `reqwest` | `0.12.23` | 信息源与服务提供方的 HTTPS 请求 |
| `sqlx` | `0.8.6` | SQLite 访问与迁移 |
| `serde` | `1.0.219` | 序列化 |
| `serde_json` | `1.0.142` | JSON 载荷 |
| `chrono` | `0.4.41` | UTC 时间戳与时间间隔 |
| `chrono-tz` | `0.10.4` | `Asia/Shanghai` 调度计算 |
| `uuid` | `1.17.0` | 公开标识符与内部标识符 |
| `url` | `2.5.4` | URL 规范化 |
| `scraper` | `0.23.1` | HTML 解析 |
| `regex` | `1.11.1` | 规则预筛选 |
| `sha2` | `0.10.9` | 内容指纹与令牌指纹 |
| `argon2` | `0.5.3` | 配对码哈希 |
| `rand` | `0.9.2` | 安全随机值 |
| `base64` | `0.22.1` | FCM/JWT 载荷编码 |
| `jsonwebtoken` | `9.3.1` | 用于 FCM OAuth 的服务账号 JWT |
| `lettre` | `0.11.17` | 通过 TLS 使用 Gmail SMTP |
| `tower-http` | `0.6.6` | 请求追踪、超时与 CORS 策略 |
| `governor` | `0.8.1` | API 与手动检查的速率限制 |
| `tracing` | `0.1.41` | 结构化日志 |
| `tracing-subscriber` | `0.3.19` | 日志格式化与过滤 |
| `thiserror` | `2.0.12` | 类型化领域错误 |
| `anyhow` | `1.0.98` | 顶层任务错误上下文 |
| `dotenvy` | `0.15.7` | 加载本地开发环境 |
| `clap` | `4.5.45` | 用于配对码和迁移的管理员 CLI |
| `zeroize` | `1.8.1` | 清理内存中的密钥材料 |
| `wiremock` | `0.6.5` | 服务提供方与信息源的集成测试 |
| `proptest` | `1.7.0` | 解析器与去重属性测试 |

功能选项：

- `reqwest`：`rustls-tls`、`json`、`gzip`、`brotli`、`deflate`。
- `sqlx`：`sqlite`、`runtime-tokio-rustls`、`macros`、`migrate`、`chrono`、`uuid`。
- `lettre`：`tokio1-rustls-tls`、`builder`。
- `jsonwebtoken`：使用 RSA SHA-256 为 FCM 服务账号断言签名。
- v1 不包含浏览器运行时。

## 7. 后端服务提供方契约

### 7.1 模型中转服务

- 协议：兼容 OpenAI 的 Chat Completions。
- 方法：`POST {MODEL_BASE_URL}/v1/chat/completions`。
- 模型：`gpt-5.6-terra`。
- 请求参数：`temperature: 0`、有界的 `max_tokens: 1200`；如果中转服务支持，则使用 JSON 响应模式。
- 提示词要求只返回一个符合 `BACKEND_STRUCTURE.md` 中分类器模式的 JSON 对象。
- 后端必须验证响应，并拒绝格式错误或包含额外内容的响应。
- 如果服务提供方超时、返回非 2xx 响应或生成无效 JSON，则本次运行记为部分失败，且不得根据该次分类发送预警。

### 7.2 Gmail SMTP 邮件服务

- 主机：`smtp.gmail.com`。
- 端口：`465`。
- 安全：隐式 TLS。
- 身份验证：`xingheluqi56@gmail.com` + Gmail 应用专用密码。
- 收件人：`2331613886@qq.com`。
- 不使用端口 25。

### 7.3 Firebase Cloud Messaging 推送服务

- 协议：FCM HTTP v1。
- 端点：`https://fcm.googleapis.com/v1/projects/{project_id}/messages:send`。
- 目标：一个设备注册令牌。
- 身份验证：由 Firebase 服务账号 JSON 文件生成的 OAuth 2.0 访问令牌。
- 服务账号文件以只读方式挂载，绝不提交到版本库。

## 8. 部署工具

| 工具 | 精确版本 |
|---|---:|
| Docker Engine | `27.5.1` |
| Docker Compose 插件 | `2.35.1` |
| Nginx | `1.28.0` |
| Certbot | `5.4.0` |
| SQLite 运行时 | `3.46.1` |

TLS 使用采用短期配置文件的 Let’s Encrypt IP 地址证书。证书有效期约为六天，必须至少提前 48 小时自动续期。正式签发生产证书之前，必须先使用测试端点验证续期任务。

## 9. 代码库目录结构

```text
.
├── PRD.md
├── APP_FLOW.md
├── TECH_STACK.md
├── FRONTEND_GUIDELINES.md
├── BACKEND_STRUCTURE.md
├── IMPLEMENTATION_PLAN.md
├── server/
│   ├── Cargo.toml
│   ├── Cargo.lock
│   ├── rust-toolchain.toml
│   ├── migrations/
│   └── src/
├── android/
│   ├── settings.gradle.kts
│   ├── gradle/libs.versions.toml
│   ├── app/
│   └── gradle/wrapper/
├── contracts/
│   └── openapi.yaml
└── infra/
    ├── Dockerfile
    ├── docker-compose.yml
    ├── nginx.conf
    ├── certbot/
    └── systemd/
```

## 10. 官方参考资料

- [Compose 中的 Material 3](https://developer.android.com/develop/ui/compose/designsystems/material3)
- [Android 版 Firebase Cloud Messaging](https://firebase.google.com/docs/cloud-messaging/android/receive-messages)
- [FCM HTTP v1 API](https://firebase.google.com/docs/cloud-messaging/send/v1-api)
- [Android 网络安全配置](https://developer.android.com/privacy-and-security/security-config)
- [Let’s Encrypt IP 地址证书](https://letsencrypt.org/2026/01/15/6day-and-ip-general-availability.html)

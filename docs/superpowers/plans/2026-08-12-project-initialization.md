# 项目初始化与依赖安装实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking progress.

**Goal:** 按 `IMPLEMENTATION_PLAN.md` 步骤 1 初始化 Rust 服务端、Android 客户端、契约/部署/脚本目录，并锁定 TECH_STACK.md 中规定的工具链和直接依赖。

**Architecture:** 根目录保留规范和状态文件；`server/` 使用 Cargo 管理 Rust 二进制和锁文件；`android/` 使用 Gradle Wrapper、Android Gradle Plugin 和版本目录管理 Kotlin/Compose 应用；其余目录先建立可扩展骨架。

**Tech Stack:** Rust `1.89.0` / edition 2024、Cargo、Android Gradle Plugin `8.13.0`、Gradle `8.13`、Kotlin `2.2.21`、Compose BOM `2025.10.00`、compile/target SDK `36`、min SDK `28`。

## Global Constraints

- 所有直接依赖使用 TECH_STACK.md 中的精确版本；Cargo 清单使用 `=版本` 约束，Gradle 版本目录使用固定版本字符串。
- Rust 工具链文件必须设置 `channel = "1.89.0"`、`profile = "minimal"` 和目标 `aarch64-unknown-linux-gnu`。
- Android 包名和 applicationId 必须为 `com.xingheluqi.codexresetwatch`。
- Android 生产网络安全配置必须禁用明文流量；真实 `google-services.json` 只能位于本地且被 `.gitignore` 忽略。
- 不写入真实凭据，不修改与步骤 1 无关的业务规范。
- 每个任务以对应的命令验证，并在完成后更新根目录 `progress.txt`。

## 文件结构

- `server/`：Rust 服务端 Cargo 包、工具链锁定、源码入口和依赖锁文件。
- `android/`：Gradle 工程、版本目录、应用模块、Manifest、网络安全资源和最小可编译 Activity。
- `contracts/`、`infra/`、`scripts/`：后续 API、部署和自动化脚本的空骨架目录。
- `.gitignore`：凭据、构建产物、Gradle/Rust 缓存、SQLite 和 TLS 私钥的保护规则。
- `progress.txt`：记录步骤 1 的完成情况和实际验证结果。

---

### Task 1: 创建仓库骨架

**Files:**
- Create: `server/`
- Create: `android/`
- Create: `contracts/`
- Create: `infra/`
- Create: `scripts/`
- Modify: `.gitignore` only if a required ignore pattern is missing

**Interfaces:**
- Produces the directories consumed by the Rust, Android, contract, infrastructure, and script tasks.

- [x] **Step 1: Create the directories**

```bash
mkdir -p server android contracts infra scripts
```

- [x] **Step 2: Verify the root skeleton and ignore rules**

```bash
find server android contracts infra scripts -maxdepth 1 -type d -print
git check-ignore -q .env server/target android/.gradle android/local.properties google-services.json
```

Expected: all five directories exist and every sensitive/build path is ignored.

### Task 2: Initialize and lock the Rust workspace

**Files:**
- Create: `server/Cargo.toml`
- Create: `server/Cargo.lock`
- Create: `server/rust-toolchain.toml`
- Create: `server/src/main.rs`

**Interfaces:**
- Produces a Cargo package named `codex-reset-watch` with an async `main() -> anyhow::Result<()>` entry point and all TECH_STACK.md Rust direct dependencies.

- [x] **Step 1: Install and select the pinned Rust toolchain**

```bash
rustup toolchain install 1.89.0 --profile minimal
rustup target add --toolchain 1.89.0 aarch64-unknown-linux-gnu
```

- [x] **Step 2: Generate the Cargo package with Cargo**

```bash
cargo init --bin --name codex-reset-watch server
```

- [x] **Step 3: Add the exact runtime, API, persistence, parsing, security, notification, and test dependencies**

```bash
cd server
cargo add axum@=0.8.4
cargo add tokio@=1.47.1 --features macros,rt-multi-thread
cargo add reqwest@=0.12.23 --no-default-features --features rustls-tls,json,gzip,brotli,deflate
cargo add sqlx@=0.8.6 --no-default-features --features sqlite,runtime-tokio-rustls,macros,migrate,chrono,uuid
cargo add serde@=1.0.219 --features derive
cargo add serde_json@=1.0.142
cargo add chrono@=0.4.41 --features serde
cargo add chrono-tz@=0.10.4
cargo add uuid@=1.17.0 --features serde,v4
cargo add url@=2.5.4
cargo add scraper@=0.23.1
cargo add regex@=1.11.1
cargo add sha2@=0.10.9
cargo add argon2@=0.5.3
cargo add rand@=0.9.2
cargo add base64@=0.22.1
cargo add jsonwebtoken@=9.3.1 --no-default-features --features use_pem
cargo add lettre@=0.11.17 --no-default-features --features tokio1-rustls-tls,builder
cargo add tower-http@=0.6.6
cargo add governor@=0.8.1
cargo add tracing@=0.1.41
cargo add tracing-subscriber@=0.3.19
cargo add thiserror@=2.0.12
cargo add anyhow@=1.0.98
cargo add dotenvy@=0.15.7
cargo add clap@=4.5.45 --features derive
cargo add zeroize@=1.8.1
cargo add --dev wiremock@=0.6.5
cargo add --dev proptest@=1.7.0
cd ..
```

- [x] **Step 4: Set the edition, toolchain, and minimal entry point**

Write `rust-toolchain.toml` with `channel = "1.89.0"`, `profile = "minimal"`, and the `aarch64-unknown-linux-gnu` target; set `edition = "2024"` and package version `1.0.0`; implement the Tokio entry point returning `Ok(())`.

- [x] **Step 5: Resolve and lock the dependency graph**

```bash
cd server
cargo generate-lockfile
cargo check --locked
cargo test --locked
cd ..
```

Expected: Cargo uses Rust 1.89.0, check and test exit 0, and `server/Cargo.lock` is present.

### Task 3: Initialize and lock the Android workspace

**Files:**
- Create: `android/settings.gradle.kts`
- Create: `android/build.gradle.kts`
- Create: `android/gradle/libs.versions.toml`
- Create: `android/gradle.properties`
- Create: `android/gradle/wrapper/gradle-wrapper.properties`
- Create: `android/app/build.gradle.kts`
- Create: `android/app/src/main/AndroidManifest.xml`
- Create: `android/app/src/main/java/com/xingheluqi/codexresetwatch/MainActivity.kt`
- Create: `android/app/src/main/res/xml/network_security_config.xml`
- Create: `android/app/src/main/res/values/strings.xml`
- Create: `android/app/src/main/res/values/themes.xml`

**Interfaces:**
- Produces an Android application module using `com.xingheluqi.codexresetwatch`, Compose Material 3, Ktor, DataStore, coroutines, serialization, Firebase Messaging, and the specified test dependencies.

- [x] **Step 1: Generate the basic Gradle project and wrapper**

```bash
cd android
gradle init --type basic --dsl kotlin --project-name codex-reset-watch --no-incubating
gradle wrapper --gradle-version 8.13
cd ..
```

- [x] **Step 2: Configure the version catalog and plugin repositories**

Declare AGP `8.13.0`, Kotlin and Compose compiler `2.2.21`, all fixed library versions from TECH_STACK.md, Compose BOM `2025.10.00`, and Firebase BOM `34.3.0`; use Google, Maven Central, and Gradle Plugin Portal repositories.

- [x] **Step 3: Configure the Android application module**

Set namespace/applicationId `com.xingheluqi.codexresetwatch`, compileSdk/targetSdk `36`, minSdk `28`, NDK `27.2.12479018`, Java/Kotlin target `21`, Compose and Kotlin serialization plugins, and all specified implementation/test dependencies.

- [x] **Step 4: Add the minimal secure application shell**

Add a launcher `MainActivity` using Compose Material 3, a manifest with `android:networkSecurityConfig="@xml/network_security_config"` and `android:usesCleartextTraffic="false"`, and the XML network-security/theme/string resources needed for a debug build. Do not add credentials or `google-services.json`.

- [x] **Step 5: Resolve dependencies and build/test the app**

```bash
cd android
./gradlew --version
./gradlew :app:assembleDebug
./gradlew :app:testDebugUnitTest
cd ..
```

Expected: Gradle reports `8.13`, the debug APK assembles, and the unit-test task exits 0.

### Task 4: Record progress and final verification

**Files:**
- Modify: `progress.txt`

**Interfaces:**
- Records the completed “项目初始化与依赖安装” milestone, exact verification commands, any environment caveats, and the next implementation step.

- [x] **Step 1: Verify the generated tree and repository diff**

```bash
find server android contracts infra scripts -maxdepth 2 -type d -print
git status --short
```

- [x] **Step 2: Update `progress.txt`**

Move the current stage past project initialization, add “项目初始化与依赖安装” under completed work, record successful Rust/Gradle verification commands, and set the next step to `IMPLEMENTATION_PLAN.md` task 2.1.

- [x] **Step 3: Re-read the status and run the final checks**

```bash
sed -n '1,260p' progress.txt
git diff --check
git status --short
```

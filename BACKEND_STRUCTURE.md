# Codex 重置预警——后端结构文档

**后端语言：** Rust `1.89.0`

**异步运行时：** Tokio

**HTTP 框架：** Axum `0.8.4`

**数据库：** SQLite `3.46.1`，通过 SQLx `0.8.6` 访问

**公共基础 URL：** `https://<fixed-public-ip>/api`

## 1. 后端职责

后端是唯一可信的决策方，负责：

- 按北京时间调度任务。
- 获取各数据源并执行数据源专用解析。
- 维护增量游标和内容指纹。
- 对证据进行标准化。
- 执行规则预筛选和模型辅助分类。
- 完成预警聚类、去重、级别升级和通知幂等控制。
- 使用 SQLite 持久化数据并执行 30 天保留策略。
- 完成 Android 设备配对和 FCM 令牌管理。
- 通过 FCM 和 Gmail 投递通知。
- 向 Android 应用提供 REST API。

后端不得向 Android 应用暴露数据源凭据、模型凭据、Gmail 凭据或 Firebase 服务账号数据。

## 2. 运行时模块

```text
server/src/
├── main.rs                 # Process startup and dependency wiring
├── config.rs               # Environment parsing and validation
├── error.rs                # Public and internal error types
├── time.rs                 # UTC storage and Asia/Shanghai schedule logic
├── api/
│   ├── mod.rs
│   ├── auth.rs
│   ├── home.rs
│   ├── pairing.rs
│   ├── alerts.rs
│   ├── runs.rs
│   ├── preferences.rs
│   └── health.rs
├── auth/
│   ├── pairing_codes.rs
│   ├── device_tokens.rs
│   └── middleware.rs
├── scheduler/
│   ├── mod.rs
│   ├── hourly.rs
│   └── manual.rs
├── sources/
│   ├── mod.rs
│   ├── x_thsottiaux.rs
│   ├── quota_forecast.rs
│   ├── openai_status.rs
│   ├── http.rs
│   └── normalize.rs
├── classification/
│   ├── rules.rs
│   ├── prompt.rs
│   ├── model_client.rs
│   ├── schema.rs
│   └── clustering.rs
├── notifications/
│   ├── mod.rs
│   ├── fcm.rs
│   ├── email.rs
│   └── dedup.rs
├── db/
│   ├── mod.rs
│   ├── models.rs
│   └── queries.rs
└── retention.rs
```

## 3. 数据库约定

- 每个 SQLite 连接都必须启用外键。
- 所有内部 ID 均为 UUID v4 字符串，并以 `TEXT` 存储。
- 所有时间戳均为 UTC ISO 8601 字符串，并以 `TEXT` 存储；API 序列化结果必须包含 `Z` 后缀或显式时区偏移量。
- 布尔值使用 `INTEGER NOT NULL CHECK (value IN (0,1))`。
- 枚举使用 `TEXT`，由应用程序进行校验，并在可行时添加数据库 `CHECK` 约束。
- 所有表都包含 `created_at`；可变表还必须包含 `updated_at`。
- 原始外部载荷仅保留 30 天。
- 密钥不得以明文形式存储在 SQLite 中。

## 4. 数据库模式

### 4.1 `app_config`

单例配置表。仅允许存在 `id = 1` 的记录。

| 列名 | SQLite 类型 | 约束 | 说明 |
|---|---|---|---|
| `id` | `INTEGER` | PK, CHECK `id = 1` | 单例键 |
| `timezone` | `TEXT` | NOT NULL, default `Asia/Shanghai` | 显示和调度时区 |
| `monitor_start_minute` | `INTEGER` | NOT NULL, default `480` | 从午夜起算的分钟数，`480` 表示 08:00 |
| `monitor_end_minute` | `INTEGER` | NOT NULL, default `1380` | 从午夜起算的分钟数，`1380` 表示 23:00 |
| `catchup_start_minute` | `INTEGER` | NOT NULL, default `0` | 前一日补查的起始时间 |
| `history_retention_days` | `INTEGER` | NOT NULL, default `30` | 数据保留天数 |
| `created_at` | `TEXT` | NOT NULL | UTC 时间戳 |
| `updated_at` | `TEXT` | NOT NULL | UTC 时间戳 |

### 4.2 `source_cursors`

每个数据源类别对应一条记录。

| 列名 | SQLite 类型 | 约束 | 说明 |
|---|---|---|---|
| `source_id` | `TEXT` | PK | `x_thsottiaux`、`quota_forecast` 或 `openai_status` |
| `cursor_kind` | `TEXT` | NOT NULL | 时间戳、外部 ID 或页面标记 |
| `cursor_value` | `TEXT` | NULL | 数据源专用游标 |
| `last_success_at` | `TEXT` | NULL | 最近一次成功获取的时间 |
| `last_attempt_at` | `TEXT` | NULL | 最近一次尝试的时间 |
| `health_state` | `TEXT` | NOT NULL | `healthy`、`degraded` 或 `failed` |
| `last_http_status` | `INTEGER` | NULL | 最近一次 HTTP 响应状态码 |
| `last_error` | `TEXT` | NULL | 已脱敏的错误摘要 |
| `created_at` | `TEXT` | NOT NULL | UTC 时间戳 |
| `updated_at` | `TEXT` | NOT NULL | UTC 时间戳 |

### 4.3 `source_items`

标准化的数据源记录。数据源条目的身份不可变，但其内容可以更新。

| 列名 | SQLite 类型 | 约束 | 说明 |
|---|---|---|---|
| `id` | `TEXT` | PK | 内部 UUID |
| `source_id` | `TEXT` | FK → `source_cursors.source_id` | 数据源类别 |
| `external_id` | `TEXT` | NOT NULL | X 帖子 ID、Status 事件或更新 ID，或者预测快照键 |
| `canonical_url` | `TEXT` | NOT NULL | 标准化后的公开 URL |
| `parent_external_id` | `TEXT` | NULL | 直接父级回复或事件 ID |
| `thread_root_external_id` | `TEXT` | NULL | 适用时的 X 主题串根节点 |
| `published_at` | `TEXT` | NULL | 数据源发布时间 |
| `fetched_at` | `TEXT` | NOT NULL | 后端抓取时间 |
| `raw_payload_json` | `TEXT` | NOT NULL | 已脱敏的原始响应 |
| `normalized_text` | `TEXT` | NOT NULL | 用于搜索和分类的文本 |
| `content_hash` | `TEXT` | NOT NULL | 标准化内容的 SHA-256 值 |
| `is_public` | `INTEGER` | NOT NULL | 符合条件的记录必须为 `1` |
| `is_official_authority` | `INTEGER` | NOT NULL | 若来自允许的官方权威来源则为真 |
| `created_at` | `TEXT` | NOT NULL | UTC 时间戳 |
| `updated_at` | `TEXT` | NOT NULL | UTC 时间戳 |

唯一约束：使用 `(source_id, external_id)`，并使用 `(source_id, content_hash)` 对内容未变化的记录进行去重。

### 4.4 `forecast_observations`

从额度预测站点解析得到的结构化观测记录。

| 列名 | SQLite 类型 | 约束 | 说明 |
|---|---|---|---|
| `id` | `TEXT` | PK | 内部 UUID |
| `source_item_id` | `TEXT` | FK → `source_items.id` | 负责承载该观测的数据源记录 |
| `probability_basis_points` | `INTEGER` | NULL, 0–10000 | `82%` 对应 `8200` |
| `window_start` | `TEXT` | NULL | 预测窗口起始时间，使用 UTC |
| `window_end` | `TEXT` | NULL | 预测窗口结束时间，使用 UTC |
| `display_timezone` | `TEXT` | NOT NULL | 数据源时区或 `Asia/Shanghai` |
| `raw_value_text` | `TEXT` | NULL | 原始显示的预测值 |
| `observed_at` | `TEXT` | NOT NULL | 抓取时间戳 |
| `created_at` | `TEXT` | NOT NULL | UTC 时间戳 |

### 4.5 `signal_clusters`

合并相关数据源条目并防止重复预警的逻辑事件。

| 列名 | SQLite 类型 | 约束 | 说明 |
|---|---|---|---|
| `id` | `TEXT` | PK | 对外公开的 `alert_id` |
| `cluster_key` | `TEXT` | UNIQUE, NOT NULL | 稳定的标准化事件键 |
| `current_level` | `TEXT` | NOT NULL | `none`、`B` 或 `A` |
| `current_state` | `TEXT` | NOT NULL | `active`、`superseded`、`expired` 或 `withdrawn` |
| `window_start` | `TEXT` | NULL | 当前预计窗口的 UTC 起始时间 |
| `window_end` | `TEXT` | NULL | 当前预计窗口的 UTC 结束时间 |
| `first_seen_at` | `TEXT` | NOT NULL | 首次出现合格证据的时间 |
| `last_updated_at` | `TEXT` | NOT NULL | 最近一次出现实质性证据的时间 |
| `latest_evidence_summary` | `TEXT` | NOT NULL | 四行摘要的证据文本 |
| `recommendation` | `TEXT` | NOT NULL | `consume_quota` 或 `continue_observing` |
| `conflict_state` | `TEXT` | NOT NULL | `none`、`official_overrides_forecast` 或 `forecast_supports_official` |
| `created_at` | `TEXT` | NOT NULL | UTC 时间戳 |
| `updated_at` | `TEXT` | NOT NULL | UTC 时间戳 |

### 4.6 `evidence`

附属于逻辑预警的证据条目。

| 列名 | SQLite 类型 | 约束 | 说明 |
|---|---|---|---|
| `id` | `TEXT` | PK | 内部 UUID |
| `cluster_id` | `TEXT` | FK → `signal_clusters.id` | 预警聚类 |
| `source_item_id` | `TEXT` | FK → `source_items.id` | 数据源条目 |
| `evidence_type` | `TEXT` | NOT NULL | `official_post`、`reply_context`、`status_event` 或 `forecast_observation` |
| `quote_text` | `TEXT` | NOT NULL | 简短的来源引文 |
| `context_text` | `TEXT` | NULL | 解释所需的直接上下文 |
| `source_url` | `TEXT` | NOT NULL | 标准公开链接 |
| `published_at` | `TEXT` | NULL | 数据源时间 |
| `captured_at` | `TEXT` | NOT NULL | 后端抓取时间 |
| `relevance` | `TEXT` | NOT NULL | `primary`、`supporting` 或 `conflicting` |
| `evidence_hash` | `TEXT` | NOT NULL | 去重指纹 |
| `created_at` | `TEXT` | NOT NULL | UTC 时间戳 |

### 4.7 `classification_runs`

每次规则或模型分类尝试都必须可审计。

| 列名 | SQLite 类型 | 约束 | 说明 |
|---|---|---|---|
| `id` | `TEXT` | PK | 内部 UUID |
| `run_id` | `TEXT` | FK → `run_executions.id` | 触发本次分类的运行记录 |
| `cluster_id` | `TEXT` | FK → `signal_clusters.id`, NULL | 最终产生的预警；若无则为空 |
| `classifier_version` | `TEXT` | NOT NULL | `classifier-v1` |
| `model_provider` | `TEXT` | NOT NULL | `openai_compatible` |
| `model_name` | `TEXT` | NOT NULL | `gpt-5.6-terra` |
| `rule_prefilter_result` | `TEXT` | NOT NULL | `candidate`、`ignored` 或 `invalid` |
| `decision` | `TEXT` | NOT NULL | `A`、`B` 或 `none` |
| `confidence_basis_points` | `INTEGER` | NULL, 0–10000 | 模型置信度，不作为面向用户的概率 |
| `window_start` | `TEXT` | NULL | 模型输出的 UTC 起始时间 |
| `window_end` | `TEXT` | NULL | 模型输出的 UTC 结束时间 |
| `reason_json` | `TEXT` | NOT NULL | 经过校验的结构化判定依据 |
| `raw_response_hash` | `TEXT` | NOT NULL | 不存储可能包含密钥的原始提示词，仅保留哈希 |
| `created_at` | `TEXT` | NOT NULL | UTC 时间戳 |

### 4.8 `notifications`

具有幂等性的通知投递尝试记录。

| 列名 | SQLite 类型 | 约束 | 说明 |
|---|---|---|---|
| `id` | `TEXT` | PK | 内部 UUID |
| `cluster_id` | `TEXT` | FK → `signal_clusters.id` | 预警 |
| `channel` | `TEXT` | NOT NULL | `fcm` 或 `email` |
| `event_type` | `TEXT` | NOT NULL | `initial`、`upgrade` 或 `material_update` |
| `dedupe_key` | `TEXT` | UNIQUE, NOT NULL | 聚类 + 渠道 + 事件类型 + 版本 |
| `payload_json` | `TEXT` | NOT NULL | 已脱敏的通知载荷 |
| `delivery_state` | `TEXT` | NOT NULL | `pending`、`sent`、`failed` 或 `suppressed` |
| `provider_message_id` | `TEXT` | NULL | FCM 或 SMTP 提供方消息 ID |
| `attempt_count` | `INTEGER` | NOT NULL, default 0 | 投递尝试次数 |
| `last_attempt_at` | `TEXT` | NULL | UTC 时间戳 |
| `sent_at` | `TEXT` | NULL | UTC 时间戳 |
| `last_error` | `TEXT` | NULL | 已脱敏的错误信息 |
| `created_at` | `TEXT` | NOT NULL | UTC 时间戳 |
| `updated_at` | `TEXT` | NOT NULL | UTC 时间戳 |

### 4.9 `device_bindings`

v1 仅允许一台活跃设备；数据库模式允许未来支持多台设备，但应用逻辑必须强制最多只有一条活跃记录。

| 列名 | SQLite 类型 | 约束 | 说明 |
|---|---|---|---|
| `id` | `TEXT` | PK | 设备 UUID |
| `access_token_hash` | `TEXT` | UNIQUE, NOT NULL | 不透明持有者令牌的 SHA-256 值 |
| `fcm_token_ciphertext` | `BLOB` | NOT NULL | 加密后的 FCM 令牌 |
| `fcm_token_nonce` | `BLOB` | NOT NULL | 加密随机数 |
| `device_model` | `TEXT` | NOT NULL | `OnePlus 13` |
| `android_version` | `TEXT` | NOT NULL | 设备操作系统版本 |
| `app_version` | `TEXT` | NOT NULL | 应用版本 |
| `paired_at` | `TEXT` | NOT NULL | UTC 时间戳 |
| `last_seen_at` | `TEXT` | NOT NULL | UTC 时间戳 |
| `revoked_at` | `TEXT` | NULL | UTC 时间戳 |
| `created_at` | `TEXT` | NOT NULL | UTC 时间戳 |
| `updated_at` | `TEXT` | NOT NULL | UTC 时间戳 |

### 4.10 `pairing_codes`

供管理员使用的一次性配对码。

| 列名 | SQLite 类型 | 约束 | 说明 |
|---|---|---|---|
| `id` | `TEXT` | PK | 内部 UUID |
| `code_hash` | `TEXT` | NOT NULL | 配对码的 Argon2id 哈希值 |
| `created_at` | `TEXT` | NOT NULL | UTC 时间戳 |
| `expires_at` | `TEXT` | NOT NULL | 创建时间 + 10 分钟 |
| `consumed_at` | `TEXT` | NULL | 使用时间戳 |
| `attempt_count` | `INTEGER` | NOT NULL, default 0 | 失败尝试次数 |
| `created_by` | `TEXT` | NOT NULL | `admin_cli` |

### 4.11 `run_executions`

每次计划运行或手动运行对应一条记录。

| 列名 | SQLite 类型 | 约束 | 说明 |
|---|---|---|---|
| `id` | `TEXT` | PK | 运行 UUID |
| `run_kind` | `TEXT` | NOT NULL | `scheduled`、`catchup`、`bootstrap` 或 `manual` |
| `scheduled_for` | `TEXT` | NULL | 计划的北京时间事件，以 UTC 序列化存储 |
| `window_start` | `TEXT` | NOT NULL | 扫描窗口起始时间，使用 UTC |
| `window_end` | `TEXT` | NOT NULL | 扫描窗口结束时间，使用 UTC |
| `started_at` | `TEXT` | NOT NULL | UTC 时间戳 |
| `finished_at` | `TEXT` | NULL | UTC 时间戳 |
| `result` | `TEXT` | NOT NULL | `running`、`no_alert`、`alert`、`partial_failure` 或 `failed` |
| `source_status_json` | `TEXT` | NOT NULL | 各数据源的计数和状态 |
| `created_alert_count` | `INTEGER` | NOT NULL, default 0 | 新建预警数量 |
| `updated_alert_count` | `INTEGER` | NOT NULL, default 0 | 实质性更新数量 |
| `error_summary` | `TEXT` | NULL | 已脱敏的摘要 |
| `created_at` | `TEXT` | NOT NULL | UTC 时间戳 |
| `updated_at` | `TEXT` | NOT NULL | UTC 时间戳 |

### 4.12 `user_preferences`

单例个人偏好设置。

| 列名 | SQLite 类型 | 约束 | 说明 |
|---|---|---|---|
| `id` | `INTEGER` | PK, CHECK `id = 1` | 单例键 |
| `push_enabled` | `INTEGER` | NOT NULL, default 1 | FCM 投递偏好 |
| `email_enabled` | `INTEGER` | NOT NULL, default 1 | 邮件投递偏好 |
| `created_at` | `TEXT` | NOT NULL | UTC 时间戳 |
| `updated_at` | `TEXT` | NOT NULL | UTC 时间戳 |

## 5. 关系与索引

```text
source_cursors 1 ─── * source_items
source_items   1 ─── * forecast_observations
source_items   1 ─── * evidence
signal_clusters 1 ── * evidence
signal_clusters 1 ── * classification_runs
signal_clusters 1 ── * notifications
run_executions 1 ─── * classification_runs
device_bindings 1 ── * notification delivery targets (logical, not FK)
```

必须创建以下索引：

- `source_items(source_id, external_id)`，唯一索引。
- `source_items(source_id, published_at)`。
- `source_items(content_hash)`。
- `evidence(cluster_id, captured_at)`。
- `signal_clusters(current_state, last_updated_at)`。
- `notifications(cluster_id, channel, event_type)`。
- `run_executions(started_at)`。
- `pairing_codes(expires_at, consumed_at)`。

## 6. 认证与配对

### 6.1 生成配对码

服务器管理员 CLI 使用大写字母和数字生成八位配对码，并排除容易混淆的字符 `O`、`0`、`I` 和 `1`。CLI 只显示一次明文配对码，服务器仅存储其 Argon2id 哈希值。

- 有效期：10 分钟。
- 使用次数：恰好一次。
- 失败尝试：每个配对码最多五次，达到上限后锁定该配对码。
- v1 只能存在一台活跃的已绑定设备。
- 如果已有设备完成绑定，则必须先撤销旧设备，才能接受新的配对请求。

### 6.2 申请配对

`POST /api/v1/pair/claim` 校验配对码，创建一个随机的 32 字节不透明访问令牌，仅存储其 SHA-256 哈希值，存储加密后的 FCM 令牌，将配对码标记为已使用，并且只通过 HTTPS 返回一次明文访问令牌。

Android 应用将该令牌存入由 Android Keystore 支持的加密存储空间。应用在受保护的 API 请求中通过 `Authorization: Bearer <token>` 发送令牌。

### 6.3 撤销设备

`POST /api/v1/device/revoke` 将设备标记为已撤销。只有在服务器确认操作成功后，应用才删除本地令牌。再次绑定时必须使用新的配对码。

### 6.4 请求中间件

- `GET /healthz` 和 `GET /readyz` 无需认证。
- 除 `/api/v1/pair/claim` 外，所有 `/api/v1/*` 端点都需要持有者令牌。
- 无效、缺失、已撤销或未知的令牌返回 `401` 和 `AUTH_INVALID`。
- 配对尝试按 IP 和配对码 ID 进行速率限制。
- 手动检查每 5 分钟最多接受一次请求。

## 7. REST API 契约

### 7.1 通用请求头

```http
Content-Type: application/json
Accept: application/json
Authorization: Bearer <device_access_token>
X-Client-Version: 1.0.0
X-Request-Id: <uuid>
```

### 7.2 通用成功响应封装

```json
{
  "data": {},
  "request_id": "uuid"
}
```

### 7.3 通用错误响应封装

```json
{
  "error": {
    "code": "AUTH_INVALID",
    "message": "The device credential is invalid or revoked.",
    "retryable": false
  },
  "request_id": "uuid"
}
```

其中，`AUTH_INVALID` 表示设备凭据无效或已被撤销，且该错误不可重试。

### 7.4 健康检查

#### `GET /healthz`

仅用于存活检查。进程存活时返回 `200`。

```json
{"status":"ok"}
```

#### `GET /readyz`

仅当 SQLite 可读且必需配置均已加载时返回 `200`；否则返回 `503`。

### 7.5 设备配对

#### `POST /api/v1/pair/claim`

请求：

```json
{
  "pairing_code": "AB7K9Q2M",
  "fcm_token": "<fcm-registration-token>",
  "device_model": "OnePlus 13",
  "android_version": "15",
  "app_version": "1.0.0"
}
```

响应 `201`：

```json
{
  "data": {
    "device_id": "uuid",
    "access_token": "opaque-token",
    "home": {}
  },
  "request_id": "uuid"
}
```

### 7.6 首页

#### `GET /api/v1/home`

返回当前生效的预警、运行状态、下次运行时间、数据源健康状态和通知偏好状态。

响应字段：

```json
{
  "data": {
    "service_state": "normal",
    "last_check_at": "2026-08-12T10:00:00+08:00",
    "next_check_at": "2026-08-12T11:00:00+08:00",
    "timezone": "Asia/Shanghai",
    "monitor_window": "08:00-23:00",
    "active_alert": null,
    "source_health": [
      {"source_id":"x_thsottiaux","state":"healthy","last_success_at":"...","last_error":null},
      {"source_id":"quota_forecast","state":"healthy","last_success_at":"...","last_error":null},
      {"source_id":"openai_status","state":"healthy","last_success_at":"...","last_error":null}
    ],
    "preferences": {"push_enabled":true,"email_enabled":true}
  },
  "request_id":"uuid"
}
```

### 7.7 预警

#### `GET /api/v1/alerts?level=all&state=all&cursor=<cursor>&limit=20`

返回 30 天保留窗口内最多 20 条预警。允许的级别为 `all`、`A`、`B`；允许的状态为 `all`、`active`、`archived`、`withdrawn`。

#### `GET /api/v1/alerts/{alert_id}`

返回：

- 预警级别和状态。
- 四行摘要。
- 建议。
- 按北京时间显示的窗口。
- 首次发现时间和最近更新时间。
- 冲突状态。
- 证据列表。
- 标准来源 URL。

### 7.8 运行记录

#### `GET /api/v1/runs?from=<iso>&to=<iso>&cursor=<cursor>&limit=50`

返回最近 30 天内的运行记录。`from` 和 `to` 为可选参数，并按北京时间的 ISO 值解释。

#### `GET /api/v1/runs/{run_id}`

返回运行窗口、运行类型、结果、持续时间、各数据源状态、错误摘要和受影响的预警 ID。

### 7.9 手动检查

#### `POST /api/v1/checks`

请求：

```json
{"reason":"user_requested"}
```

响应 `202`：

```json
{
  "data": {"run_id":"uuid","status":"queued"},
  "request_id":"uuid"
}
```

如果已有兼容的手动运行正在执行，则返回 `202` 和该运行现有的 `run_id`。如果五分钟速率限制仍然生效，则返回 `429` 和 `retry_after_seconds`。

### 7.10 设备心跳和 FCM 刷新

#### `POST /api/v1/device/heartbeat`

请求：

```json
{
  "fcm_token":"<current-token>",
  "device_model":"OnePlus 13",
  "android_version":"15",
  "app_version":"1.0.0"
}
```

成功时返回 `204`。

### 7.11 撤销设备

#### `POST /api/v1/device/revoke`

撤销当前已认证的设备。成功时返回 `204`。

### 7.12 偏好设置

#### `GET /api/v1/preferences`

返回推送和邮件是否启用的标志。

#### `PATCH /api/v1/preferences`

请求：

```json
{"push_enabled":true,"email_enabled":false}
```

字段均为可选，但必须至少提供一个字段。响应返回完整的偏好设置对象。

### 7.13 测试通知

#### `POST /api/v1/notifications/test`

请求：

```json
{"channels":["fcm","email"]}
```

响应分别报告每个渠道的结果。测试通知不得创建预警，也不得改变去重状态。

## 8. 调度器与扫描流程

1. 使用 `chrono-tz` 计算下一个北京时间整点边界。
2. 08:00 创建补查运行，覆盖前一日 00:00 至当日 08:00 的时间窗口。
3. 其他所有计划整点运行均从最近一次成功的数据源游标扫描至计划执行时间。
4. 在设定超时时间的前提下，独立获取三个数据源。
5. 标准化数据源条目并计算指纹。
6. 执行确定性预筛选规则。
7. 根据标准化的事件身份和直接主题串上下文对候选项进行聚类。
8. 仅对通过预筛选的候选项调用模型。
9. 校验分类器 JSON，并执行确定性的 A/B 后置校验。
10. 每次逻辑更新均在一个事务内持久化证据、分类结果、预警状态转换和运行状态。
11. 创建具有幂等性的 FCM 和邮件投递记录。
12. 在数据库事务之外发送通知。
13. 标记投递状态；仅当数据源获取和持久化均成功后，才推进对应的数据源游标。

## 9. 分类器契约

模型必须且只能返回一个 JSON 对象：

```json
{
  "decision":"A|B|none",
  "future_signal":true,
  "official_signal":true,
  "quota_object":"Codex usage limits",
  "window_start":"2026-08-13T01:00:00Z",
  "window_end":"2026-08-13T03:00:00Z",
  "evidence_sentence":"One concise sentence.",
  "recommendation":"consume_quota|continue_observing",
  "ambiguity":"none|object|scope|time|context",
  "forecast_probability_basis_points":8200,
  "conflict":"none|official_overrides_forecast|forecast_supports_official",
  "reason_codes":["official_future_quota_signal"]
}
```

除非官方的未来额度对象、24 小时时间窗口和直接上下文要求全部满足，否则确定性后置校验器必须拒绝 `decision=A`。当 `future_signal=false` 时，后置校验器必须拒绝 `none` 之外的任何判定。

## 10. 数据保留与清理

每天北京时间 03:30 执行数据保留清理：

- 删除超过 30 天的 `source_items`、`forecast_observations`、`evidence`、`classification_runs`、`notifications` 和 `run_executions`。
- 仅当 `signal_clusters` 的最近更新时间超过 30 天，且没有保留期内的证据引用它们时，才删除这些记录。
- 删除已使用或已过期超过 24 小时的配对码。
- 仅在每周低流量维护窗口执行 SQLite 真空整理；不得每小时执行。

## 11. 安全要求

- Nginx 监听 80 和 443 端口；Rust 仅监听本机回环地址。
- 80 端口仅用于 ACME 验证，并将其他所有流量重定向至 HTTPS。
- 443 端口使用固定 IPv4 地址证书。
- SSH 仅允许使用运维人员的密钥，并在支持时使用非默认端口。
- 任何 API 端点都不得暴露 SQLite、FCM 服务账号 JSON、Gmail 应用专用密码和模型 API 密钥。
- 日志必须对持有者令牌、FCM 令牌、SMTP 凭据、模型密钥和配对码进行脱敏。
- 数据源文本属于不可信输入，在分类器提示词中必须将其作为数据封装。
- API 响应必须包含请求 ID，但不得包含内部堆栈跟踪。

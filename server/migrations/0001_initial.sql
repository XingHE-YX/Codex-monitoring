CREATE TABLE app_config (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    timezone TEXT NOT NULL DEFAULT 'Asia/Shanghai',
    monitor_start_minute INTEGER NOT NULL DEFAULT 480 CHECK (monitor_start_minute BETWEEN 0 AND 1439),
    monitor_end_minute INTEGER NOT NULL DEFAULT 1380 CHECK (monitor_end_minute BETWEEN 0 AND 1439),
    catchup_start_minute INTEGER NOT NULL DEFAULT 0 CHECK (catchup_start_minute BETWEEN 0 AND 1439),
    history_retention_days INTEGER NOT NULL DEFAULT 30 CHECK (history_retention_days > 0),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE source_cursors (
    source_id TEXT PRIMARY KEY CHECK (source_id IN ('x_thsottiaux', 'quota_forecast', 'openai_status')),
    cursor_kind TEXT NOT NULL,
    cursor_value TEXT,
    last_success_at TEXT,
    last_attempt_at TEXT,
    health_state TEXT NOT NULL CHECK (health_state IN ('healthy', 'degraded', 'failed')),
    last_http_status INTEGER CHECK (last_http_status IS NULL OR last_http_status BETWEEN 100 AND 599),
    last_error TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE source_items (
    id TEXT PRIMARY KEY,
    source_id TEXT NOT NULL REFERENCES source_cursors(source_id) ON DELETE RESTRICT,
    external_id TEXT NOT NULL,
    canonical_url TEXT NOT NULL,
    parent_external_id TEXT,
    thread_root_external_id TEXT,
    published_at TEXT,
    fetched_at TEXT NOT NULL,
    raw_payload_json TEXT NOT NULL,
    normalized_text TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    is_public INTEGER NOT NULL CHECK (is_public IN (0, 1)),
    is_official_authority INTEGER NOT NULL CHECK (is_official_authority IN (0, 1)),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (source_id, external_id),
    UNIQUE (source_id, content_hash)
);

CREATE TABLE forecast_observations (
    id TEXT PRIMARY KEY,
    source_item_id TEXT NOT NULL REFERENCES source_items(id) ON DELETE CASCADE,
    probability_basis_points INTEGER CHECK (probability_basis_points IS NULL OR probability_basis_points BETWEEN 0 AND 10000),
    window_start TEXT,
    window_end TEXT,
    display_timezone TEXT NOT NULL,
    raw_value_text TEXT,
    observed_at TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE signal_clusters (
    id TEXT PRIMARY KEY,
    cluster_key TEXT NOT NULL UNIQUE,
    current_level TEXT NOT NULL CHECK (current_level IN ('none', 'B', 'A')),
    current_state TEXT NOT NULL CHECK (current_state IN ('active', 'superseded', 'expired', 'withdrawn')),
    window_start TEXT,
    window_end TEXT,
    first_seen_at TEXT NOT NULL,
    last_updated_at TEXT NOT NULL,
    latest_evidence_summary TEXT NOT NULL,
    recommendation TEXT NOT NULL CHECK (recommendation IN ('consume_quota', 'continue_observing')),
    conflict_state TEXT NOT NULL CHECK (conflict_state IN ('none', 'official_overrides_forecast', 'forecast_supports_official')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE run_executions (
    id TEXT PRIMARY KEY,
    run_kind TEXT NOT NULL CHECK (run_kind IN ('scheduled', 'catchup', 'bootstrap', 'manual')),
    scheduled_for TEXT,
    window_start TEXT NOT NULL,
    window_end TEXT NOT NULL,
    started_at TEXT NOT NULL,
    finished_at TEXT,
    result TEXT NOT NULL CHECK (result IN ('running', 'no_alert', 'alert', 'partial_failure', 'failed')),
    source_status_json TEXT NOT NULL,
    created_alert_count INTEGER NOT NULL DEFAULT 0 CHECK (created_alert_count >= 0),
    updated_alert_count INTEGER NOT NULL DEFAULT 0 CHECK (updated_alert_count >= 0),
    error_summary TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE evidence (
    id TEXT PRIMARY KEY,
    cluster_id TEXT NOT NULL REFERENCES signal_clusters(id) ON DELETE CASCADE,
    source_item_id TEXT NOT NULL REFERENCES source_items(id) ON DELETE RESTRICT,
    evidence_type TEXT NOT NULL CHECK (evidence_type IN ('official_post', 'reply_context', 'status_event', 'forecast_observation')),
    quote_text TEXT NOT NULL,
    context_text TEXT,
    source_url TEXT NOT NULL,
    published_at TEXT,
    captured_at TEXT NOT NULL,
    relevance TEXT NOT NULL CHECK (relevance IN ('primary', 'supporting', 'conflicting')),
    evidence_hash TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE (cluster_id, evidence_hash)
);

CREATE TABLE classification_runs (
    id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL REFERENCES run_executions(id) ON DELETE CASCADE,
    cluster_id TEXT REFERENCES signal_clusters(id) ON DELETE SET NULL,
    classifier_version TEXT NOT NULL,
    model_provider TEXT NOT NULL CHECK (model_provider = 'openai_compatible'),
    model_name TEXT NOT NULL CHECK (model_name = 'gpt-5.6-terra'),
    rule_prefilter_result TEXT NOT NULL CHECK (rule_prefilter_result IN ('candidate', 'ignored', 'invalid')),
    decision TEXT NOT NULL CHECK (decision IN ('A', 'B', 'none')),
    confidence_basis_points INTEGER CHECK (confidence_basis_points IS NULL OR confidence_basis_points BETWEEN 0 AND 10000),
    window_start TEXT,
    window_end TEXT,
    reason_json TEXT NOT NULL,
    raw_response_hash TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE notifications (
    id TEXT PRIMARY KEY,
    cluster_id TEXT NOT NULL REFERENCES signal_clusters(id) ON DELETE CASCADE,
    channel TEXT NOT NULL CHECK (channel IN ('fcm', 'email')),
    event_type TEXT NOT NULL CHECK (event_type IN ('initial', 'upgrade', 'material_update')),
    dedupe_key TEXT NOT NULL UNIQUE,
    payload_json TEXT NOT NULL,
    delivery_state TEXT NOT NULL CHECK (delivery_state IN ('pending', 'sent', 'failed', 'suppressed')),
    provider_message_id TEXT,
    attempt_count INTEGER NOT NULL DEFAULT 0 CHECK (attempt_count >= 0),
    last_attempt_at TEXT,
    sent_at TEXT,
    last_error TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE device_bindings (
    id TEXT PRIMARY KEY,
    access_token_hash TEXT NOT NULL UNIQUE,
    fcm_token_ciphertext BLOB NOT NULL,
    fcm_token_nonce BLOB NOT NULL,
    device_model TEXT NOT NULL,
    android_version TEXT NOT NULL,
    app_version TEXT NOT NULL,
    paired_at TEXT NOT NULL,
    last_seen_at TEXT NOT NULL,
    revoked_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE pairing_codes (
    id TEXT PRIMARY KEY,
    code_hash TEXT NOT NULL,
    created_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    consumed_at TEXT,
    attempt_count INTEGER NOT NULL DEFAULT 0 CHECK (attempt_count BETWEEN 0 AND 5),
    created_by TEXT NOT NULL CHECK (created_by = 'admin_cli')
);

CREATE TABLE user_preferences (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    push_enabled INTEGER NOT NULL DEFAULT 1 CHECK (push_enabled IN (0, 1)),
    email_enabled INTEGER NOT NULL DEFAULT 1 CHECK (email_enabled IN (0, 1)),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE UNIQUE INDEX idx_source_items_source_external ON source_items(source_id, external_id);
CREATE INDEX idx_source_items_source_published ON source_items(source_id, published_at);
CREATE INDEX idx_source_items_content_hash ON source_items(content_hash);
CREATE INDEX idx_evidence_cluster_captured ON evidence(cluster_id, captured_at);
CREATE INDEX idx_signal_clusters_state_updated ON signal_clusters(current_state, last_updated_at);
CREATE INDEX idx_notifications_cluster_channel_event ON notifications(cluster_id, channel, event_type);
CREATE INDEX idx_run_executions_started ON run_executions(started_at);
CREATE INDEX idx_pairing_codes_expiry_consumed ON pairing_codes(expires_at, consumed_at);
CREATE UNIQUE INDEX idx_one_active_device ON device_bindings((1)) WHERE revoked_at IS NULL;

INSERT INTO app_config (
    id,
    timezone,
    monitor_start_minute,
    monitor_end_minute,
    catchup_start_minute,
    history_retention_days,
    created_at,
    updated_at
) VALUES (
    1,
    'Asia/Shanghai',
    480,
    1380,
    0,
    30,
    strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
    strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
);

INSERT INTO source_cursors (
    source_id,
    cursor_kind,
    health_state,
    created_at,
    updated_at
) VALUES
    ('x_thsottiaux', 'external_id', 'degraded', strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    ('quota_forecast', 'observation', 'degraded', strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    ('openai_status', 'external_id', 'degraded', strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));

INSERT INTO user_preferences (
    id,
    push_enabled,
    email_enabled,
    created_at,
    updated_at
) VALUES (
    1,
    1,
    1,
    strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
    strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
);

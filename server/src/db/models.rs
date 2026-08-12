use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct AppConfig {
    pub id: i64,
    pub timezone: String,
    pub monitor_start_minute: i64,
    pub monitor_end_minute: i64,
    pub catchup_start_minute: i64,
    pub history_retention_days: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct SourceCursor {
    pub source_id: String,
    pub cursor_kind: String,
    pub cursor_value: Option<String>,
    pub last_success_at: Option<DateTime<Utc>>,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub health_state: String,
    pub last_http_status: Option<i64>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct SourceItem {
    pub id: String,
    pub source_id: String,
    pub external_id: String,
    pub canonical_url: String,
    pub parent_external_id: Option<String>,
    pub thread_root_external_id: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub fetched_at: DateTime<Utc>,
    pub raw_payload_json: String,
    pub normalized_text: String,
    pub content_hash: String,
    pub is_public: bool,
    pub is_official_authority: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct ForecastObservation {
    pub id: String,
    pub source_item_id: String,
    pub probability_basis_points: Option<i64>,
    pub window_start: Option<DateTime<Utc>>,
    pub window_end: Option<DateTime<Utc>>,
    pub display_timezone: String,
    pub raw_value_text: Option<String>,
    pub observed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct SignalCluster {
    pub id: String,
    pub cluster_key: String,
    pub current_level: String,
    pub current_state: String,
    pub window_start: Option<DateTime<Utc>>,
    pub window_end: Option<DateTime<Utc>>,
    pub first_seen_at: DateTime<Utc>,
    pub last_updated_at: DateTime<Utc>,
    pub latest_evidence_summary: String,
    pub recommendation: String,
    pub conflict_state: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct Evidence {
    pub id: String,
    pub cluster_id: String,
    pub source_item_id: String,
    pub evidence_type: String,
    pub quote_text: String,
    pub context_text: Option<String>,
    pub source_url: String,
    pub published_at: Option<DateTime<Utc>>,
    pub captured_at: DateTime<Utc>,
    pub relevance: String,
    pub evidence_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct ClassificationRun {
    pub id: String,
    pub run_id: String,
    pub cluster_id: Option<String>,
    pub classifier_version: String,
    pub model_provider: String,
    pub model_name: String,
    pub rule_prefilter_result: String,
    pub decision: String,
    pub confidence_basis_points: Option<i64>,
    pub window_start: Option<DateTime<Utc>>,
    pub window_end: Option<DateTime<Utc>>,
    pub reason_json: String,
    pub raw_response_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct Notification {
    pub id: String,
    pub cluster_id: String,
    pub channel: String,
    pub event_type: String,
    pub dedupe_key: String,
    pub payload_json: String,
    pub delivery_state: String,
    pub provider_message_id: Option<String>,
    pub attempt_count: i64,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub sent_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct DeviceBinding {
    pub id: String,
    pub access_token_hash: String,
    pub fcm_token_ciphertext: Vec<u8>,
    pub fcm_token_nonce: Vec<u8>,
    pub device_model: String,
    pub android_version: String,
    pub app_version: String,
    pub paired_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct PairingCode {
    pub id: String,
    pub code_hash: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub consumed_at: Option<DateTime<Utc>>,
    pub attempt_count: i64,
    pub created_by: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct RunExecution {
    pub id: String,
    pub run_kind: String,
    pub scheduled_for: Option<DateTime<Utc>>,
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub result: String,
    pub source_status_json: String,
    pub created_alert_count: i64,
    pub updated_alert_count: i64,
    pub error_summary: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct UserPreferences {
    pub id: i64,
    pub push_enabled: bool,
    pub email_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorAdvance {
    pub source_id: String,
    pub cursor_kind: String,
    pub cursor_value: String,
    pub attempted_at: DateTime<Utc>,
    pub http_status: Option<i64>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CleanupCounts {
    pub source_items: u64,
    pub forecast_observations: u64,
    pub evidence: u64,
    pub classification_runs: u64,
    pub notifications: u64,
    pub run_executions: u64,
    pub signal_clusters: u64,
    pub pairing_codes: u64,
}

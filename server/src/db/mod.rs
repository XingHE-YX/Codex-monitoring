pub mod models;
pub mod queries;

use std::{str::FromStr, time::Duration};

use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

pub async fn connect(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let in_memory = database_url.contains(":memory:");
    let mut options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(5));
    if !in_memory {
        options = options.journal_mode(SqliteJournalMode::Wal);
    }

    SqlitePoolOptions::new()
        .max_connections(if in_memory { 1 } else { 5 })
        .connect_with(options)
        .await
}

pub async fn migrate(pool: &SqlitePool) -> anyhow::Result<()> {
    MIGRATOR.run(pool).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use sqlx::{Row, SqlitePool};

    use super::{
        connect, migrate,
        models::{CursorAdvance, Evidence, Notification, RunExecution, SignalCluster, SourceItem},
        queries::{
            cleanup_retention, insert_evidence, insert_notification_once,
            persist_source_item_and_cursor, upsert_signal_cluster,
        },
    };

    fn at(value: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(value)
            .unwrap()
            .with_timezone(&Utc)
    }

    async fn database() -> SqlitePool {
        let pool = connect("sqlite::memory:").await.unwrap();
        migrate(&pool).await.unwrap();
        pool
    }

    fn source_item(id: &str, external_id: &str, content_hash: &str) -> SourceItem {
        let now = at("2026-08-12T02:00:00Z");
        SourceItem {
            id: id.to_string(),
            source_id: "x_thsottiaux".to_string(),
            external_id: external_id.to_string(),
            canonical_url: format!("https://x.com/thsottiaux/status/{external_id}"),
            parent_external_id: None,
            thread_root_external_id: None,
            published_at: Some(at("2026-08-12T01:55:00Z")),
            fetched_at: now,
            raw_payload_json: "{}".to_string(),
            normalized_text: "future Codex usage limit reset".to_string(),
            content_hash: content_hash.to_string(),
            is_public: true,
            is_official_authority: true,
            created_at: now,
            updated_at: now,
        }
    }

    fn cursor(value: &str) -> CursorAdvance {
        CursorAdvance {
            source_id: "x_thsottiaux".to_string(),
            cursor_kind: "external_id".to_string(),
            cursor_value: value.to_string(),
            attempted_at: at("2026-08-12T02:00:00Z"),
            http_status: Some(200),
        }
    }

    fn cluster(id: &str, level: &str) -> SignalCluster {
        let now = at("2026-08-12T02:00:00Z");
        SignalCluster {
            id: id.to_string(),
            cluster_key: "codex-quota-2026-08-13".to_string(),
            current_level: level.to_string(),
            current_state: "active".to_string(),
            window_start: Some(at("2026-08-13T01:00:00Z")),
            window_end: Some(at("2026-08-13T03:00:00Z")),
            first_seen_at: now,
            last_updated_at: now,
            latest_evidence_summary: "official future quota signal".to_string(),
            recommendation: "continue_observing".to_string(),
            conflict_state: "none".to_string(),
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn migration_creates_all_tables_defaults_and_foreign_keys() {
        let pool = database().await;
        let table_names = sqlx::query_scalar::<_, String>(
            "SELECT name FROM sqlite_master \
             WHERE type = 'table' AND name NOT LIKE '_sqlx_%' \
             ORDER BY name",
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(
            table_names,
            vec![
                "app_config",
                "classification_runs",
                "device_bindings",
                "evidence",
                "forecast_observations",
                "notifications",
                "pairing_codes",
                "run_executions",
                "signal_clusters",
                "source_cursors",
                "source_items",
                "user_preferences",
            ]
        );

        let foreign_keys = sqlx::query_scalar::<_, i64>("PRAGMA foreign_keys")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(foreign_keys, 1);

        let row = sqlx::query("SELECT timezone, monitor_start_minute, monitor_end_minute, history_retention_days FROM app_config WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(row.get::<String, _>("timezone"), "Asia/Shanghai");
        assert_eq!(row.get::<i64, _>("monitor_start_minute"), 480);
        assert_eq!(row.get::<i64, _>("monitor_end_minute"), 1380);
        assert_eq!(row.get::<i64, _>("history_retention_days"), 30);
    }

    #[tokio::test]
    async fn source_item_uniqueness_and_cursor_advance_are_transactional() {
        let pool = database().await;
        let item = source_item("item-1", "post-1", "hash-1");
        persist_source_item_and_cursor(&pool, &item, &cursor("post-1"))
            .await
            .unwrap();

        let cursor_value = sqlx::query_scalar::<_, String>(
            "SELECT cursor_value FROM source_cursors WHERE source_id = 'x_thsottiaux'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(cursor_value, "post-1");

        let duplicate = source_item("item-2", "post-1", "hash-2");
        let result = sqlx::query(
            "INSERT INTO source_items (id, source_id, external_id, canonical_url, published_at, fetched_at, raw_payload_json, normalized_text, content_hash, is_public, is_official_authority, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&duplicate.id)
        .bind(&duplicate.source_id)
        .bind(&duplicate.external_id)
        .bind(&duplicate.canonical_url)
        .bind(duplicate.published_at)
        .bind(duplicate.fetched_at)
        .bind(&duplicate.raw_payload_json)
        .bind(&duplicate.normalized_text)
        .bind(&duplicate.content_hash)
        .bind(duplicate.is_public)
        .bind(duplicate.is_official_authority)
        .bind(duplicate.created_at)
        .bind(duplicate.updated_at)
        .execute(&pool)
        .await;

        assert!(result.unwrap_err().as_database_error().is_some());
    }

    #[tokio::test]
    async fn alert_upsert_preserves_identity_and_evidence_enforces_foreign_keys() {
        let pool = database().await;
        let item = source_item("item-1", "post-1", "hash-1");
        persist_source_item_and_cursor(&pool, &item, &cursor("post-1"))
            .await
            .unwrap();

        let inserted = upsert_signal_cluster(&pool, &cluster("alert-1", "B"))
            .await
            .unwrap();
        let updated = upsert_signal_cluster(&pool, &cluster("different-id", "A"))
            .await
            .unwrap();
        assert_eq!(inserted.id, "alert-1");
        assert_eq!(updated.id, "alert-1");
        assert_eq!(updated.current_level, "A");

        let evidence = Evidence {
            id: "evidence-1".to_string(),
            cluster_id: "alert-1".to_string(),
            source_item_id: "item-1".to_string(),
            evidence_type: "official_post".to_string(),
            quote_text: "limits will be restored tomorrow".to_string(),
            context_text: None,
            source_url: "https://x.com/thsottiaux/status/post-1".to_string(),
            published_at: item.published_at,
            captured_at: item.fetched_at,
            relevance: "primary".to_string(),
            evidence_hash: "evidence-hash-1".to_string(),
            created_at: item.created_at,
        };
        insert_evidence(&pool, &evidence).await.unwrap();

        let mut orphan = evidence.clone();
        orphan.id = "evidence-2".to_string();
        orphan.cluster_id = "unknown-alert".to_string();
        orphan.evidence_hash = "evidence-hash-2".to_string();
        assert!(
            insert_evidence(&pool, &orphan)
                .await
                .unwrap_err()
                .as_database_error()
                .is_some()
        );
    }

    #[tokio::test]
    async fn notification_dedupe_key_accepts_one_logical_record() {
        let pool = database().await;
        upsert_signal_cluster(&pool, &cluster("alert-1", "A"))
            .await
            .unwrap();
        let now = at("2026-08-12T02:00:00Z");
        let notification = Notification {
            id: "notification-1".to_string(),
            cluster_id: "alert-1".to_string(),
            channel: "fcm".to_string(),
            event_type: "initial".to_string(),
            dedupe_key: "alert-1:fcm:initial:v1".to_string(),
            payload_json: "{}".to_string(),
            delivery_state: "pending".to_string(),
            provider_message_id: None,
            attempt_count: 0,
            last_attempt_at: None,
            sent_at: None,
            last_error: None,
            created_at: now,
            updated_at: now,
        };

        assert!(
            insert_notification_once(&pool, &notification)
                .await
                .unwrap()
        );
        let mut duplicate = notification.clone();
        duplicate.id = "notification-2".to_string();
        assert!(!insert_notification_once(&pool, &duplicate).await.unwrap());

        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM notifications")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn retention_cleanup_removes_old_runs_and_consumed_pairing_codes() {
        let pool = database().await;
        let old = at("2026-07-01T00:00:00Z");
        let run = RunExecution {
            id: "run-old".to_string(),
            run_kind: "scheduled".to_string(),
            scheduled_for: Some(old),
            window_start: old,
            window_end: old,
            started_at: old,
            finished_at: Some(old),
            result: "no_alert".to_string(),
            source_status_json: "{}".to_string(),
            created_alert_count: 0,
            updated_alert_count: 0,
            error_summary: None,
            created_at: old,
            updated_at: old,
        };
        sqlx::query("INSERT INTO run_executions (id, run_kind, scheduled_for, window_start, window_end, started_at, finished_at, result, source_status_json, created_alert_count, updated_alert_count, error_summary, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(&run.id)
            .bind(&run.run_kind)
            .bind(run.scheduled_for)
            .bind(run.window_start)
            .bind(run.window_end)
            .bind(run.started_at)
            .bind(run.finished_at)
            .bind(&run.result)
            .bind(&run.source_status_json)
            .bind(run.created_alert_count)
            .bind(run.updated_alert_count)
            .bind(&run.error_summary)
            .bind(run.created_at)
            .bind(run.updated_at)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO pairing_codes (id, code_hash, created_at, expires_at, consumed_at, attempt_count, created_by) VALUES ('pair-old', 'hash', ?, ?, ?, 0, 'admin_cli')")
            .bind(old)
            .bind(old)
            .bind(old)
            .execute(&pool)
            .await
            .unwrap();

        let counts = cleanup_retention(
            &pool,
            at("2026-07-13T00:00:00Z"),
            at("2026-08-11T00:00:00Z"),
        )
        .await
        .unwrap();
        assert_eq!(counts.run_executions, 1);
        assert_eq!(counts.pairing_codes, 1);
    }
}

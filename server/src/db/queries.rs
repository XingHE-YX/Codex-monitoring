use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

use super::models::{
    CleanupCounts, CursorAdvance, Evidence, Notification, SignalCluster, SourceItem,
};

pub async fn persist_source_item_and_cursor(
    pool: &SqlitePool,
    item: &SourceItem,
    cursor: &CursorAdvance,
) -> Result<(), sqlx::Error> {
    let mut transaction = pool.begin().await?;

    sqlx::query(
        "INSERT INTO source_items (id, source_id, external_id, canonical_url, parent_external_id, thread_root_external_id, published_at, fetched_at, raw_payload_json, normalized_text, content_hash, is_public, is_official_authority, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
         ON CONFLICT(source_id, external_id) DO UPDATE SET \
           canonical_url = excluded.canonical_url, \
           parent_external_id = excluded.parent_external_id, \
           thread_root_external_id = excluded.thread_root_external_id, \
           published_at = excluded.published_at, \
           fetched_at = excluded.fetched_at, \
           raw_payload_json = excluded.raw_payload_json, \
           normalized_text = excluded.normalized_text, \
           content_hash = excluded.content_hash, \
           is_public = excluded.is_public, \
           is_official_authority = excluded.is_official_authority, \
           updated_at = excluded.updated_at",
    )
    .bind(&item.id)
    .bind(&item.source_id)
    .bind(&item.external_id)
    .bind(&item.canonical_url)
    .bind(&item.parent_external_id)
    .bind(&item.thread_root_external_id)
    .bind(item.published_at)
    .bind(item.fetched_at)
    .bind(&item.raw_payload_json)
    .bind(&item.normalized_text)
    .bind(&item.content_hash)
    .bind(item.is_public)
    .bind(item.is_official_authority)
    .bind(item.created_at)
    .bind(item.updated_at)
    .execute(&mut *transaction)
    .await?;

    let update = sqlx::query(
        "UPDATE source_cursors SET \
           cursor_kind = ?, cursor_value = ?, last_success_at = ?, last_attempt_at = ?, \
           health_state = 'healthy', last_http_status = ?, last_error = NULL, updated_at = ? \
         WHERE source_id = ?",
    )
    .bind(&cursor.cursor_kind)
    .bind(&cursor.cursor_value)
    .bind(cursor.attempted_at)
    .bind(cursor.attempted_at)
    .bind(cursor.http_status)
    .bind(cursor.attempted_at)
    .bind(&cursor.source_id)
    .execute(&mut *transaction)
    .await?;

    if update.rows_affected() != 1 {
        return Err(sqlx::Error::RowNotFound);
    }

    transaction.commit().await
}

pub async fn upsert_signal_cluster(
    pool: &SqlitePool,
    cluster: &SignalCluster,
) -> Result<SignalCluster, sqlx::Error> {
    sqlx::query_as::<_, SignalCluster>(
        "INSERT INTO signal_clusters (id, cluster_key, current_level, current_state, window_start, window_end, first_seen_at, last_updated_at, latest_evidence_summary, recommendation, conflict_state, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
         ON CONFLICT(cluster_key) DO UPDATE SET \
           current_level = excluded.current_level, \
           current_state = excluded.current_state, \
           window_start = excluded.window_start, \
           window_end = excluded.window_end, \
           last_updated_at = excluded.last_updated_at, \
           latest_evidence_summary = excluded.latest_evidence_summary, \
           recommendation = excluded.recommendation, \
           conflict_state = excluded.conflict_state, \
           updated_at = excluded.updated_at \
         RETURNING *",
    )
    .bind(&cluster.id)
    .bind(&cluster.cluster_key)
    .bind(&cluster.current_level)
    .bind(&cluster.current_state)
    .bind(cluster.window_start)
    .bind(cluster.window_end)
    .bind(cluster.first_seen_at)
    .bind(cluster.last_updated_at)
    .bind(&cluster.latest_evidence_summary)
    .bind(&cluster.recommendation)
    .bind(&cluster.conflict_state)
    .bind(cluster.created_at)
    .bind(cluster.updated_at)
    .fetch_one(pool)
    .await
}

pub async fn insert_evidence(pool: &SqlitePool, evidence: &Evidence) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO evidence (id, cluster_id, source_item_id, evidence_type, quote_text, context_text, source_url, published_at, captured_at, relevance, evidence_hash, created_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&evidence.id)
    .bind(&evidence.cluster_id)
    .bind(&evidence.source_item_id)
    .bind(&evidence.evidence_type)
    .bind(&evidence.quote_text)
    .bind(&evidence.context_text)
    .bind(&evidence.source_url)
    .bind(evidence.published_at)
    .bind(evidence.captured_at)
    .bind(&evidence.relevance)
    .bind(&evidence.evidence_hash)
    .bind(evidence.created_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn insert_notification_once(
    pool: &SqlitePool,
    notification: &Notification,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO notifications (id, cluster_id, channel, event_type, dedupe_key, payload_json, delivery_state, provider_message_id, attempt_count, last_attempt_at, sent_at, last_error, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
         ON CONFLICT(dedupe_key) DO NOTHING",
    )
    .bind(&notification.id)
    .bind(&notification.cluster_id)
    .bind(&notification.channel)
    .bind(&notification.event_type)
    .bind(&notification.dedupe_key)
    .bind(&notification.payload_json)
    .bind(&notification.delivery_state)
    .bind(&notification.provider_message_id)
    .bind(notification.attempt_count)
    .bind(notification.last_attempt_at)
    .bind(notification.sent_at)
    .bind(&notification.last_error)
    .bind(notification.created_at)
    .bind(notification.updated_at)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() == 1)
}

pub async fn cleanup_retention(
    pool: &SqlitePool,
    retention_cutoff: DateTime<Utc>,
    pairing_cutoff: DateTime<Utc>,
) -> Result<CleanupCounts, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    let forecast_observations =
        sqlx::query("DELETE FROM forecast_observations WHERE created_at < ?")
            .bind(retention_cutoff)
            .execute(&mut *transaction)
            .await?
            .rows_affected();
    let evidence = sqlx::query("DELETE FROM evidence WHERE created_at < ?")
        .bind(retention_cutoff)
        .execute(&mut *transaction)
        .await?
        .rows_affected();
    let classification_runs = sqlx::query("DELETE FROM classification_runs WHERE created_at < ?")
        .bind(retention_cutoff)
        .execute(&mut *transaction)
        .await?
        .rows_affected();
    let notifications = sqlx::query("DELETE FROM notifications WHERE created_at < ?")
        .bind(retention_cutoff)
        .execute(&mut *transaction)
        .await?
        .rows_affected();
    let source_items = sqlx::query(
        "DELETE FROM source_items WHERE created_at < ? \
         AND NOT EXISTS (SELECT 1 FROM evidence WHERE evidence.source_item_id = source_items.id)",
    )
    .bind(retention_cutoff)
    .execute(&mut *transaction)
    .await?
    .rows_affected();
    let run_executions = sqlx::query(
        "DELETE FROM run_executions WHERE created_at < ? \
         AND NOT EXISTS (SELECT 1 FROM classification_runs WHERE classification_runs.run_id = run_executions.id)",
    )
    .bind(retention_cutoff)
    .execute(&mut *transaction)
    .await?
    .rows_affected();
    let signal_clusters = sqlx::query(
        "DELETE FROM signal_clusters WHERE last_updated_at < ? \
         AND NOT EXISTS (SELECT 1 FROM evidence WHERE evidence.cluster_id = signal_clusters.id)",
    )
    .bind(retention_cutoff)
    .execute(&mut *transaction)
    .await?
    .rows_affected();
    let pairing_codes = sqlx::query(
        "DELETE FROM pairing_codes WHERE \
         (consumed_at IS NOT NULL AND consumed_at < ?) OR expires_at < ?",
    )
    .bind(pairing_cutoff)
    .bind(pairing_cutoff)
    .execute(&mut *transaction)
    .await?
    .rows_affected();

    transaction.commit().await?;

    Ok(CleanupCounts {
        source_items,
        forecast_observations,
        evidence,
        classification_runs,
        notifications,
        run_executions,
        signal_clusters,
        pairing_codes,
    })
}

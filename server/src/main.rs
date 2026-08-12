#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = codex_reset_watch::config::Config::load()?;
    let pool = codex_reset_watch::db::connect(&config.database_url).await?;
    codex_reset_watch::db::migrate(&pool).await?;
    Ok(())
}

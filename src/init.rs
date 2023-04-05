use color_eyre::eyre::{eyre, Result};
use meilisearch_sdk::indexes::Index;
use meilisearch_sdk::{client::Client as MeiliClient, settings::Settings};
use once_cell::sync::OnceCell;

use crate::config::MeilisearchConfig;

pub static MEILICLIENT_THREAD_INDEX: OnceCell<Index> = OnceCell::new();

pub async fn meilisearch(meili: &MeilisearchConfig) -> Result<()> {
    let mclient = MeiliClient::new(&meili.api_endpoint, &meili.api_key);
    let msettings = Settings::new()
        .with_searchable_attributes(["title", "messages", "tags", "author_id", "id"])
        .with_filterable_attributes(["timestamp", "tags"])
        .with_sortable_attributes(["timestamp"])
        .with_distinct_attribute("title");
    let threads_index_db = {
        let index_uid = "threads";
        if let Ok(res) = mclient.get_index(index_uid).await {
            res
        } else {
            let task = mclient.create_index(index_uid, None).await?;
            let task = task.wait_for_completion(&mclient, None, None).await?;
            let task = task
                .try_make_index(&mclient)
                .ok()
                .ok_or_else(|| eyre!("Can't make index"))?;
            task.set_settings(&msettings).await?;
            task
        }
    };

    MEILICLIENT_THREAD_INDEX
        .set(threads_index_db)
        .ok()
        .ok_or_else(|| eyre!("Can't cache the meiliclient index"))?;
    Ok(())
}

pub fn tracing() -> Result<()> {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    let fmt_layer = fmt::layer().with_target(false);
    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();
    Ok(())
}

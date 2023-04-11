use color_eyre::eyre::{eyre, Result};
use meilisearch_sdk::{client::Client as MeiliClient, indexes::Index, settings::Settings};
use once_cell::sync::OnceCell;
use sysinfo::{System, SystemExt};
use tokio::time::{sleep, Duration, Instant};
use tracing::{event, Level};

use crate::config::MeilisearchConfig;

pub static MEILICLIENT_THREAD_INDEX: OnceCell<Index> = OnceCell::new();

async fn wait_for_meili_server(url: &str, timeout: Duration) -> Result<()> {
    let client = reqwest::Client::new();
    let start = Instant::now();

    loop {
        if let Ok(response) = client.get(url).send().await && response.status().is_success() {
            break;
        }

        if start.elapsed() > timeout {
            return Err(eyre!("Timeout waiting for server."));
        }

        sleep(Duration::from_millis(200)).await;
    }

    Ok(())
}

pub async fn meilisearch(meili: &MeilisearchConfig) -> Result<()> {
    let mut system = System::new_all();
    system.refresh_processes();
    if system
        .processes_by_name("meilisearch")
        .peekable()
        .peek()
        .is_none()
    {
        // Get executable path and parent dir
        let exec_path = std::env::current_exe()?;
        let exec_parent_dir = exec_path
            .parent()
            .ok_or_else(|| eyre!("Failed to get parent dir of self"))?;

        let mut server_cmd = meili.server_cmd.clone();
        // Add db-path if not specified
        if !&server_cmd.contains(&"--db-path".to_owned()) {
            let db_dir = exec_parent_dir
                .join("data.ms")
                .to_string_lossy()
                .to_string();
            server_cmd.extend(vec!["--db-path".to_owned(), db_dir]);
        }

        // Start the meilisearch server
        std::process::Command::new(&server_cmd[0])
            .args(&server_cmd[1..])
            .args(["--log-level", "WARN"])
            .args(["--master-key", &meili.master_key])
            .spawn()?;

        wait_for_meili_server(&meili.api_endpoint, Duration::from_secs(15)).await?;
    } else {
        event!(Level::WARN, "Meilisearch server is already running");
    }

    let mclient = MeiliClient::new(&meili.api_endpoint, Some(&meili.master_key));
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

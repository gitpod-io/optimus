use crate::config::MeilisearchConfig;
use color_eyre::eyre::{eyre, Result};
use meilisearch_sdk::{client::Client, indexes::Index, settings::Settings};
use once_cell::sync::OnceCell;
use sysinfo::{System, SystemExt};
use tokio::time::{sleep, Duration, Instant};
pub static MEILICLIENT_THREAD_INDEX: OnceCell<Index> = OnceCell::new();

fn remove_scheme_from_url(url_str: &str) -> Result<String> {
    let url = url::Url::parse(url_str)?;
    let host = url.host_str().ok_or_else(|| eyre!("Invalid host {url}"))?;
    let path = url.path().trim_end_matches('/');
    let port = url.port().ok_or_else(|| eyre!("Invalid port {url}"))?;

    Ok(format!("{host}{path}:{port}"))
}

pub async fn meilisearch(meili: &MeilisearchConfig) -> Result<()> {
    let meili_api_endpoint = &meili.api_endpoint;

    let meili_api_endpoint_without_scheme = remove_scheme_from_url(meili_api_endpoint)?;
    let meili_api_endpoint_without_scheme = meili_api_endpoint_without_scheme.as_str();

    let mclient = Client::new(meili_api_endpoint, Some(&meili.master_key));

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

        // Update PATH
        if let Ok(value) = std::env::var("PATH") {
            std::env::set_var(
                "PATH",
                format!("{}:{value}", exec_parent_dir.to_string_lossy()),
            );
        }

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
            .args(["--http-addr", meili_api_endpoint_without_scheme])
            .args(["--master-key", &meili.master_key])
            .current_dir(exec_parent_dir)
            .spawn()?;

        // Await for the server to be fully started
        let start = Instant::now();
        while !mclient.is_healthy().await {
            if start.elapsed() > Duration::from_secs(1000) {
                return Err(eyre!("Timeout waiting for server."));
            }
            println!("Awaiting for Meilisearch to be up ...");
            sleep(Duration::from_millis(300)).await;
        }
    } else {
        eprintln!("Meilisearch server is already running");
    }

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

    println!("Meilisearch is now fully up and healthy.");

    Ok(())
}

pub fn tracing() -> Result<()> {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    let fmt_layer = fmt::layer().with_target(true).pretty();
    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("warn"))?;

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();
    Ok(())
}

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use walkdir::WalkDir;

use crate::model::common::resource::ResourcePath;
use crate::model::po::resource::ResourcePo;
use crate::state::AppState;
use crate::util::time::UnixTimestampSecs;

pub async fn purge_orphaned_resources(state: Arc<AppState>) -> anyhow::Result<()> {
    let resources = {
        let mut db = state.db.acquire().await?;
        let resources = crate::storage::db::resource::all(&mut db).await?;
        resources
    };

    let f = move || purge(resources);
    let span = tracing::Span::current();
    let task = tokio::task::spawn_blocking(move || span.in_scope(f));

    let count = task.await?;

    tracing::info!("清理孤立资源文件 {count} 个");

    Ok(())
}

fn purge(resources: Vec<ResourcePo>) -> u64 {
    let time_threshold = UnixTimestampSecs::now().sub(Duration::from_secs(600));
    let mut count: u64 = 0;

    let resources = resources
        .iter()
        .map(|r| r.path.relative())
        .collect::<HashSet<_>>();

    for entry in WalkDir::new(&crate::config::get().resource.upload_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
    {
        let Some(created) = entry
            .metadata()
            .ok()
            .and_then(|m| m.created().ok())
            .and_then(UnixTimestampSecs::from_system_time)
        else {
            continue;
        };
        if created.as_i64() >= time_threshold.as_i64() {
            continue;
        }
        let Some(path) = entry.path().to_str() else {
            continue;
        };
        let path = match ResourcePath::from_absolute(&path) {
            Ok(path) => path,
            Err(err) => {
                tracing::error!("{err}");
                continue;
            }
        };
        if !resources.contains(path.relative()) {
            if let Err(e) = std::fs::remove_file(path.absolute()) {
                tracing::warn!("文件删除失败，路径：{}，错误：{e}", path.absolute());
            } else {
                count += 1;
            }
        }
    }

    count
}

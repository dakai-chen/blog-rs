mod core;
mod task;

use std::sync::{Arc, OnceLock};

use tokio_cron_scheduler::JobScheduler;

use crate::cron::core::CronTaskCollector;
use crate::state::AppState;

static SCHEDULER: OnceLock<JobScheduler> = OnceLock::new();

pub fn get() -> &'static JobScheduler {
    SCHEDULER.get().expect("定时任务未初始化")
}

pub async fn start() -> anyhow::Result<()> {
    get().start().await.map_err(From::from)
}

pub async fn shutdown() -> anyhow::Result<()> {
    get().clone().shutdown().await.map_err(From::from)
}

pub async fn init(state: Arc<AppState>) -> anyhow::Result<()> {
    let scheduler = JobScheduler::new().await?;

    task::build(state)?.register_to(&scheduler).await?;

    SCHEDULER
        .set(scheduler)
        .map_err(|_| anyhow::anyhow!("重复初始化定时任务"))
}

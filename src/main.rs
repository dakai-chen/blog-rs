#![allow(dead_code)]

mod app;
mod config;
mod context;
mod cron;
mod error;
mod jwt;
mod logger;
mod markdown;
mod middleware;
mod model;
mod response;
mod service;
mod shutdown;
mod state;
mod storage;
mod template;
mod util;
mod validator;

use std::env::VarError;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use boluo::server::{GracefulShutdown, RunError, Server};
use tokio::net::TcpListener;
use tracing_appender::non_blocking::WorkerGuard;

use crate::config::HttpConfig;
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    load_environment()?;

    initialize_config()?;

    let _guard = initialize_logging()?;

    let state = initialize_app_state().await?;

    initialize_storage(&state).await?;

    initialize_cron(&state).await?;

    let graceful = start_http_server(&state).await?;

    shutdown(state, graceful).await;

    Ok(())
}

fn load_environment() -> anyhow::Result<()> {
    dotenvy::from_path("./.env").or_else(|e| {
        // 忽略文件未找到错误
        e.not_found().then_some(()).ok_or(e)
    })?;
    Ok(())
}

fn initialize_config() -> anyhow::Result<()> {
    let mode: Option<String> = match std::env::var("app.mode") {
        Ok(val) => Some(val),
        Err(VarError::NotPresent) => None,
        Err(e) => return Err(e.into()),
    };
    config::init(mode.as_deref())?;
    shutdown::set_timeout(config::get().http.shutdown_timeout);
    Ok(())
}

fn initialize_logging() -> anyhow::Result<Option<WorkerGuard>> {
    let guard = logger::init(&config::get().logger)?;
    tracing::debug!("{}", serde_json::to_string_pretty(config::get())?);
    Ok(guard)
}

async fn initialize_app_state() -> anyhow::Result<Arc<AppState>> {
    let state = AppState::from_config(config::get()).await?;
    crate::state::global_init(state.clone())?;
    Ok(state)
}

async fn initialize_storage(state: &Arc<AppState>) -> anyhow::Result<()> {
    if config::get().database.migrations.auto_migrate {
        let mut db = state.db.acquire().await?;
        crate::storage::db::init_database_schema(&mut db).await?;
    }
    crate::storage::cache::backend::init(state.db.clone())?;
    Ok(())
}

async fn initialize_cron(state: &Arc<AppState>) -> anyhow::Result<()> {
    tracing::info!("初始化定时任务");
    cron::init(state.clone()).await?;
    cron::start().await?;
    Ok(())
}

async fn shutdown(state: Arc<AppState>, graceful: GracefulShutdown) {
    let shutdown_http = async {
        let shutdown_timeout = shutdown::timeout();
        if let Some(timeout) = shutdown_timeout {
            tracing::info!("关闭 HTTP 服务，等待活跃请求处理完成（超时时间：{timeout:?}）");
        } else {
            tracing::info!("关闭 HTTP 服务，等待活跃请求处理完成");
        }
        if graceful.shutdown(shutdown_timeout).await.is_err() {
            tracing::warn!("关闭 HTTP 服务超时");
        }
    };

    let shutdown_cron = async {
        tracing::info!("关闭定时任务（超时时间：6s）");
        match tokio::time::timeout(Duration::from_secs(6), cron::shutdown()).await {
            Err(_) => tracing::error!("关闭定时任务超时"),
            Ok(Err(e)) => tracing::error!("关闭定时任务失败：{e}"),
            Ok(_) => {}
        }
    };

    let shutdown_database = async {
        tracing::info!("关闭数据库连接池（超时时间：6s）");
        match tokio::time::timeout(Duration::from_secs(6), state.db.close()).await {
            Err(_) => tracing::error!("关闭数据库连接池超时"),
            Ok(_) => {}
        }
    };

    tokio::join!(shutdown_http, shutdown_cron);
    shutdown_database.await;

    tracing::info!("应用程序退出");
}

async fn start_http_server(state: &Arc<AppState>) -> anyhow::Result<GracefulShutdown> {
    let app = app::build(state.clone()).await?;
    let tcp = listen().await?;

    tracing::info!("HTTP 服务启动，监听地址：{}", tcp.local_addr()?);

    let graceful = match Server::new(tcp)
        .run_with_graceful_shutdown(app, shutdown::signal())
        .await
    {
        Ok(graceful) => graceful,
        Err(RunError::Listener(e, graceful)) => {
            tracing::error!("HTTP 服务监听器发生错误: {e}");
            graceful
        }
    };

    Ok(graceful)
}

async fn listen() -> anyhow::Result<TcpListener> {
    let HttpConfig {
        bind_ip, bind_port, ..
    } = config::get().http;
    Ok(TcpListener::bind(SocketAddr::from((bind_ip, bind_port))).await?)
}

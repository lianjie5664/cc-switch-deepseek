use crate::database::Database;
use crate::services::{ProxyService, UsageCache};
use std::sync::Arc;
use tokio::sync::Mutex;

/// 全局应用状态
pub struct AppState {
    pub db: Arc<Database>,
    pub proxy_service: ProxyService,
    pub usage_cache: Arc<UsageCache>,
    pub deepseek_proxy_process: Arc<Mutex<Option<tokio::process::Child>>>,
}

impl AppState {
    /// 创建新的应用状态
    pub fn new(db: Arc<Database>) -> Self {
        let proxy_service = ProxyService::new(db.clone());

        Self {
            db,
            proxy_service,
            usage_cache: Arc::new(UsageCache::new()),
            deepseek_proxy_process: Arc::new(Mutex::new(None)),
        }
    }
}

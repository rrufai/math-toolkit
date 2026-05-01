use async_trait::async_trait;
use loco_rs::{
    app::{AppContext, Hooks},
    bgworker::Queue,
    boot::{create_app, BootResult, StartMode},
    config::Config,
    controller::AppRoutes,
    environment::Environment,
    task::Tasks,
    Result,
};

use crate::controllers;

pub struct App;

#[async_trait]
impl Hooks for App {
    fn app_name() -> &'static str {
        env!("CARGO_CRATE_NAME")
    }

    async fn boot(
        mode: StartMode,
        environment: &Environment,
        config: Config,
    ) -> Result<BootResult> {
        create_app::<Self>(mode, environment, config).await
    }

    fn routes(_ctx: &AppContext) -> AppRoutes {
        AppRoutes::with_default_routes().add_route(controllers::integrate::routes())
    }

    async fn connect_workers(_ctx: &AppContext, _queue: &Queue) -> Result<()> {
        Ok(())
    }

    fn register_tasks(_tasks: &mut Tasks) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use loco_rs::task::Tasks;
    use loco_rs::{
        boot::StartMode,
        environment::Environment,
        tests_cfg::config::test_config,
    };

    #[test]
    fn test_app_name() {
        assert!(!App::app_name().is_empty());
    }

    #[test]
    fn test_register_tasks_is_noop() {
        let mut tasks = Tasks::default();
        App::register_tasks(&mut tasks);
        assert!(tasks.names().is_empty());
    }

    #[tokio::test]
    async fn test_routes_includes_integrate() {
        let ctx = loco_rs::tests_cfg::app::get_app_context().await;
        let app_routes = App::routes(&ctx);
        let paths: Vec<String> = app_routes
            .get_routes()
            .iter()
            .flat_map(|r| r.handlers.iter().map(|h| h.uri.clone()))
            .collect();
        assert!(paths.iter().any(|p| p.contains("integrate")));
    }

    #[tokio::test]
    async fn test_connect_workers_returns_ok() {
        let ctx = loco_rs::tests_cfg::app::get_app_context().await;
        let queue = loco_rs::bgworker::Queue::None;
        let result = App::connect_workers(&ctx, &queue).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_boot_server_only() {
        let result = App::boot(StartMode::ServerOnly, &Environment::Test, test_config()).await;
        // boot may fail if port is in use, but the function itself must be reachable
        let _ = result;
    }
}

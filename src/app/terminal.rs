mod component;
mod ipc;
mod layout;

use std::sync::LazyLock;

use chrono::Utc;
use dball_client::ipc::client::StateSubscriber;
use dball_client::ipc::protocol::{AppState as IpcAppState, GenerationStatus};
use dball_combora::dball::DBall;
use iocraft::prelude::*;
use layout::MainLayout;
use tokio::sync::RwLock;

fn create_default_app_state() -> IpcAppState {
    let mut app_state = IpcAppState {
        current_period: "2025084".to_owned(),
        next_period: "2025085".to_owned(),
        last_draw_time: None,
        next_draw_time: None,
        latest_ticket: None,
        pending_tickets: vec![],
        unprize_spots_count: 12,
        total_investment: 0.0,
        total_return: 0.0,
        api_status: dball_client::ipc::protocol::ApiStatusInfo {
            api_provider: "default".to_owned(),
            last_success: None,
            success_rate: 0.0,
            average_response_time: std::time::Duration::from_secs(0),
        },
        last_update: Utc::now(),
        daemon_uptime: std::time::Duration::from_secs(0),
        generation_status: GenerationStatus::Idle,
        last_generation_time: None,
    };

    // Create a default DBall instance
    let dball = DBall::new([3, 8, 15, 22, 28, 33], 12, 1).expect("Failed to create default DBall");

    app_state.latest_ticket = Some(dball);
    app_state
}

static APP_UI_STATE: LazyLock<RwLock<Option<IpcAppState>>> =
    LazyLock::new(|| RwLock::const_new(Some(create_default_app_state())));

pub async fn get_app_ui_state() -> IpcAppState {
    let state = APP_UI_STATE.read().await;
    state.clone().unwrap_or_else(create_default_app_state)
}

#[component]
pub fn DballApp(mut hooks: Hooks<'_, '_>) -> impl Into<AnyElement<'static>> {
    let mut init_once = hooks.use_state(|| false);
    if !init_once.get() {
        layout::init_logger();
        init_once.set(true);

        log::info!("TUI application starting...");
        log::info!("Logger initialized successfully");
    }

    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut should_exit = hooks.use_state(|| false);

    // Initialize IPC client and state subscriber
    hooks.use_future(async move {
        // Create state subscriber
        let subscriber = StateSubscriber::new();
        // Listen to state changes
        let mut receiver = subscriber.subscribe_to_changes();

        loop {
            if receiver.changed().await.is_ok() {
                let state = receiver.borrow().clone();
                *APP_UI_STATE.write().await = state;
                log::info!("State updated from IPC");
            } else {
                log::info!("State receiver error");
                break;
            }
        }
    });

    hooks.use_terminal_events({
        move |event| match event {
            TerminalEvent::Key(KeyEvent {
                code: KeyCode::Char('q' | 'Q') | KeyCode::Esc,
                kind,
                ..
            }) if kind != KeyEventKind::Release => {
                should_exit.set(true);
            }
            _ => {}
        }
    });

    if should_exit.get() {
        system.exit();
    }

    element! {

            MainLayout()

    }
}

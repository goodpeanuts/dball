mod components;
mod layout;

use iocraft::prelude::*;
use layout::MainLayout;
use std::time::Duration;

#[component]
pub fn DballApp(mut hooks: Hooks<'_, '_>) -> impl Into<AnyElement<'static>> {
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut should_exit = hooks.use_state(|| false);

    #[expect(clippy::infinite_loop)]
    // This future runs periodic tasks; it will exit when should_exit is set to true.
    hooks.use_future(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            // 这里可以添加定时数据更新逻辑
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

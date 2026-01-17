use iocraft::prelude::*;
use std::time::{Duration, Instant};

#[derive(Default, Props)]
pub struct LogsLayoutProps {
    pub focused: bool,
    pub list_height: u16,
}

#[component]
pub fn LogsLayout(
    _hooks: Hooks<'_, '_>,
    props: &LogsLayoutProps,
) -> impl Into<AnyElement<'static>> {
    element! {
        View(
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
        ) {
            Text(
                content: if props.focused { "Logs [FOCUS]" } else { "Logs" },
                color: if props.focused { Color::Cyan } else { Color::White },
                weight: Weight::Bold,
            )
            View(
                margin_top: 1,
                flex_direction: FlexDirection::Column,
            ) {
                LogPanel(
                    focused: props.focused,
                    list_height: props.list_height,
                )
            }
        }
    }
}

/// Log output panel component
#[derive(Default, Props)]
pub struct LogPanelProps {
    pub focused: bool,
    pub list_height: u16,
}

#[component]
pub fn LogPanel(mut hooks: Hooks<'_, '_>, props: &LogPanelProps) -> impl Into<AnyElement<'static>> {
    #[derive(Default)]
    struct LogsCache {
        lines: Vec<String>,
        total_len: usize,
    }

    let logs_cache = hooks.use_state(LogsCache::default);
    const MAX_LOGS_CACHE: usize = 500;
    let list_height = props.list_height.max(1) as usize;
    let scroll_from_bottom = hooks.use_state(|| 0usize);
    #[expect(clippy::unchecked_duration_subtraction)]
    let last_scroll_at = hooks.use_state(|| Instant::now() - Duration::from_secs(1));
    let mut focused_state = hooks.use_state(|| props.focused);

    if focused_state.get() != props.focused {
        focused_state.set(props.focused);
    }

    let mut logs_cache_for_future = logs_cache;
    let scroll_from_bottom_for_future = scroll_from_bottom;
    let focused_state_for_future = focused_state;
    hooks.use_future(async move {
        #[expect(clippy::infinite_loop)]
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            if let Ok(logs) = LOGS.lock() {
                let total_len = logs.len();
                if !focused_state_for_future.get() {
                    continue;
                }
                if total_len == logs_cache_for_future.read().total_len
                    || scroll_from_bottom_for_future.get() > 0
                {
                    continue;
                }

                let recent_logs: Vec<String> = logs
                    .iter()
                    .rev()
                    .take(MAX_LOGS_CACHE)
                    .rev()
                    .cloned()
                    .collect();

                let mut cache = logs_cache_for_future.write();
                cache.lines = recent_logs;
                cache.total_len = total_len;
            }
        }
    });

    let logs = &logs_cache.read().lines;
    let max_offset = logs.len().saturating_sub(list_height);

    hooks.use_terminal_events({
        let focused = props.focused;
        let mut scroll_from_bottom = scroll_from_bottom;
        let mut last_scroll_at = last_scroll_at;
        move |event| match event {
            TerminalEvent::Key(KeyEvent { code, kind, .. }) if kind != KeyEventKind::Release => {
                match code {
                    KeyCode::Up if focused => {
                        let now = Instant::now();
                        if now.duration_since(last_scroll_at.get()) < Duration::from_millis(30) {
                            return;
                        }
                        last_scroll_at.set(now);
                        let next = scroll_from_bottom.get().saturating_add(1);
                        scroll_from_bottom.set(next.min(max_offset));
                    }
                    KeyCode::Down if focused => {
                        let now = Instant::now();
                        if now.duration_since(last_scroll_at.get()) < Duration::from_millis(30) {
                            return;
                        }
                        last_scroll_at.set(now);
                        let next = scroll_from_bottom.get().saturating_sub(1);
                        scroll_from_bottom.set(next.min(max_offset));
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    });

    // Generate log elements
    let log_element = if logs.is_empty() {
        element! {
            Text(content: "No logs available yet...", color: Color::White)
        }
        .into()
    } else {
        let offset = scroll_from_bottom.get().min(max_offset);
        let start = logs.len().saturating_sub(list_height + offset);
        let end = (start + list_height).min(logs.len());
        let mut contents = Vec::new();
        let slice = &logs[start..end];
        for (idx, log_line) in slice.iter().enumerate() {
            let mut line = log_line.clone();
            if idx + 1 < slice.len() {
                line.push('\n');
            }
            let color = get_log_color(log_line);
            contents.push(MixedTextContent::new(line).color(color));
        }

        element! {
            MixedText(contents: contents, wrap: TextWrap::NoWrap)
        }
        .into()
    };

    element! {
        View(
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
        ) {
            Fragment(children: vec![log_element])
        }
    }
}

use std::{
    io::Write,
    sync::{LazyLock, Mutex},
};

use log::LevelFilter;

// TODO: circular buffer
pub(crate) static LOGS: LazyLock<Mutex<Vec<String>>> = LazyLock::new(|| Mutex::new(vec![]));

pub struct LogWriter;

impl Write for LogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let line = String::from_utf8_lossy(buf).to_string();
        if let Ok(mut logs) = LOGS.lock() {
            logs.push(line);
        } else {
            log::error!("Failed to acquire lock on logs");
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub fn init_logger() {
    let result = env_logger::Builder::new()
        .filter(None, LevelFilter::Info)
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .target(env_logger::Target::Pipe(Box::new(LogWriter)))
        .try_init();

    if let Ok(mut logs) = LOGS.lock() {
        if result.is_ok() {
            logs.push("[INFO] Logger system initialized successfully".to_owned());
        } else {
            logs.push("[INFO] Logger already initialized, using existing logger".to_owned());
        }
        logs.push("[INFO] TUI application starting...".to_owned());
    }
}

fn get_log_color(content: &str) -> Color {
    if content.starts_with("[ERROR]") {
        Color::Red
    } else if content.starts_with("[WARN]") {
        Color::Yellow
    } else if content.starts_with("[INFO]") {
        Color::Green
    } else if content.starts_with("[DEBUG]") {
        Color::Blue
    } else if content.starts_with("[TRACE]") {
        Color::Magenta
    } else {
        Color::White
    }
}

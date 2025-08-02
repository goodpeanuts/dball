use iocraft::prelude::*;

#[component]
pub fn LogsLayout(_hooks: Hooks<'_, '_>) -> impl Into<AnyElement<'static>> {
    element! {
        View(
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
        ) {
            LogPanel()
        }
    }
}

/// Log output panel component
#[component]
pub fn LogPanel(mut hooks: Hooks<'_, '_>) -> impl Into<AnyElement<'static>> {
    let mut logs_state = hooks.use_state(Vec::new);
    const NUMS_TO_DISPLAY: usize = 30;

    hooks.use_future(async move {
        #[expect(clippy::infinite_loop)]
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            if let Ok(logs) = LOGS.lock() {
                let recent_logs: Vec<String> = logs
                    .iter()
                    .rev()
                    .take(NUMS_TO_DISPLAY)
                    .rev()
                    .cloned()
                    .collect();
                *logs_state.write() = recent_logs;
            }
        }
    });

    let logs = logs_state.read();

    // Generate log elements
    let log_elements = if logs.is_empty() {
        vec![
            element! {
                Text(content: "No logs available yet...", color: Color::White)
            }
            .into(),
        ]
    } else {
        logs.iter()
            .map(|log_line| {
                let color = get_log_color(log_line);
                element! {
                    Text(content: log_line, color: color)
                }
                .into()
            })
            .collect::<Vec<AnyElement<'static>>>()
    };

    element! {
        View(
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
        ) {
            View(
                flex_direction: FlexDirection::Column,
                margin_top: 1,
            ) {
                Fragment(children: log_elements)
            }
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

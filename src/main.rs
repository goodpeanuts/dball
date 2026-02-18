#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

// When compiling natively:
#[cfg(not(feature = "terminal"))]
fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .with_icon(
                // NOTE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "double",
        native_options,
        Box::new(|cc| Ok(Box::new(dball::eframe::TemplateApp::new(cc)))),
    )
}

#[cfg(feature = "terminal")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use dball::terminal::DballApp;
    use dball_client::ipc::client::IpcClient;
    use iocraft::prelude::*;
    use std::io::IsTerminal as _;
    IpcClient::new().connect().await?;

    if std::io::stdout().is_terminal() {
        element!(DballApp).fullscreen().await?;
    } else {
        element!(DballApp).render_loop().await?;
    }
    Ok(())
}

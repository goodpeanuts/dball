mod app;
pub mod bench;

#[cfg(not(feature = "terminal"))]
pub use app::eframe;

#[cfg(all(feature = "terminal", not(target_arch = "wasm32")))]
pub use app::terminal;

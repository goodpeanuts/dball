mod app;
pub mod bench;

#[cfg(not(feature = "terminal"))]
pub use app::eframe;

#[cfg(feature = "terminal")]
pub use app::terminal;

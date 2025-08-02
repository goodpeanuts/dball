#[cfg(all(feature = "terminal", not(target_arch = "wasm32")))]
pub mod terminal;

#[cfg(not(feature = "terminal"))]
pub mod eframe;

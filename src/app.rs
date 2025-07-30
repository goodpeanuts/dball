#[cfg(feature = "terminal")]
pub mod terminal;

#[cfg(not(feature = "terminal"))]
pub mod eframe;

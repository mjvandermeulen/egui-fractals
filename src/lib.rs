#![warn(clippy::all, rust_2018_idioms)]

mod fractal_app;

// Idiomatic Re-exports (Optional but Common)
//   Idiomatic crates often lift heavily used items up to the crate root.
pub use fractal_app::FractalApp;

// Maybe?....
// Yes! so only these are pub:
// pub use fractal_app::paint_fractal_helpers;
pub use fractal_app::structs_and_enums;

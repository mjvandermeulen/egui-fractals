#![warn(clippy::all, rust_2018_idioms)]

mod fractal_app;

// Idiomatic Re-exports (Optional but Common)
//   Idiomatic crates often lift heavily used items up to the crate root.
pub use fractal_app::FractalApp;

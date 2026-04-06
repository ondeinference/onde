#![allow(clippy::all)]
#![allow(unexpected_cfgs)]
// flutter_rust_bridge generated code uses `!` (never type) fallback that
// became a hard error in Rust 1.82+ (edition 2024 semantics).
// This allow suppresses it for the entire bridge crate.
#![allow(dependency_on_unit_never_type_fallback)]

pub mod api;
mod frb_generated;

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}

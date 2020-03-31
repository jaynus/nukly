#![allow(
    clippy::all,
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals
)]
include!("../nuklear_bindings.rs");

// Helper impls

impl From<(f32, f32, f32, f32)> for nk_rect {
    fn from(val: (f32, f32, f32, f32)) -> Self {
        Self {
            x: val.0,
            y: val.1,
            w: val.2,
            h: val.3,
        }
    }
}

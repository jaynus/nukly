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

/*
pub pixel: *mut ::std::os::raw::c_void,
pub tex_width: ::std::os::raw::c_int,
pub tex_height: ::std::os::raw::c_int,
pub permanent: nk_allocator,
pub temporary: nk_allocator,
pub custom: nk_recti,
pub cursors: [nk_cursor; 7usize],
pub glyph_count: ::std::os::raw::c_int,
pub glyphs: *mut nk_font_glyph,
pub default_font: *mut nk_font,
pub fonts: *mut nk_font,
pub config: *mut nk_font_config,
pub font_num: ::std::os::raw::c_int,*/

impl std::fmt::Debug for nk_font_atlas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("nk_font_atlas")
            .field("pixel", &format!("{:x?}", self.pixel))
            .field("tex_width", &self.tex_width)
            .field("tex_height", &self.tex_height)
            .field("glyph_count", &self.glyph_count)
            .field("default_font", &format!("{:x?}", self.default_font))
            .field("font_num", &self.font_num)
            .field("fonts", &format!("{:x?}", self.fonts))
            .field(
                "config",
                if self.config.is_null() {
                    &"N/A"
                } else {
                    unsafe { &*self.config }
                },
            )
            .finish()
    }
}

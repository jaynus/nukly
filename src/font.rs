use crate::{alloc::Allocator, NuklearType};
use nukly_sys as sys;

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use ::{alloc::collections::Vec, core::convert::TryInto};

#[cfg(feature = "std")]
use std::{cell::RefCell, convert::TryInto, pin::Pin, sync::Arc};

#[derive(Debug, thiserror::Error)]
pub enum AtlasBakeError {
    #[error("An Unknown error occured baking the font atlas image")]
    Unknown,
    #[error(
        "Nuklear returned invalid iamge dimensions for the atlas. Got: ({}, {})",
        0.0,
        0.1
    )]
    InvalidDimensions((i32, i32)),
}

pub struct Config {
    inner: RefCell<sys::nk_font_config>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            inner: RefCell::new(sys::nk_font_config::default()),
        }
    }
}
impl Config {
    #[allow(clippy::unused_self)]
    pub fn with_glyph_range<I>(self, ranges: I) -> Self
    where
        I: IntoIterator<Item = (u32, u32)>,
    {
        let iter = ranges.into_iter();
        //iter.for_each(|item| )
        unimplemented!()
    }

    pub fn with_oversample(self, vertical: u8, horizontal: u8) -> Self {
        {
            let mut inner = self.inner.borrow_mut();

            inner.oversample_v = vertical;
            inner.oversample_h = horizontal;
        }
        self
    }

    pub fn with_size(self, size: f32) -> Self {
        self.inner.borrow_mut().size = size;
        self
    }

    pub fn with_pixel_snap(self) -> Self {
        self.inner.borrow_mut().pixel_snap = 1;
        self
    }

    pub fn with_merge_mode(self) -> Self {
        self.inner.borrow_mut().merge_mode = 1;
        self
    }
}

pub struct Font {
    inner: *mut sys::nk_font,
}
impl Font {
    fn new(inner: *mut sys::nk_font) -> Self {
        Self { inner }
    }

    pub fn handle(&self) -> *mut sys::nk_user_font {
        unsafe { &mut (*self.inner).handle as _ }
    }
}
impl NuklearType<sys::nk_font> for Font {
    fn as_ptr(&self) -> *mut sys::nk_font {
        self.inner
    }
}

pub struct Atlas {
    inner: RefCell<sys::nk_font_atlas>,
    allocator: Pin<Arc<dyn Allocator>>,
    fonts: Vec<Font>,
}
impl NuklearType<sys::nk_font_atlas> for Atlas {
    fn as_ptr(&self) -> *mut sys::nk_font_atlas {
        self.inner.as_ptr()
    }
}
impl Drop for Atlas {
    fn drop(&mut self) {
        unsafe { sys::nk_font_atlas_clear(self.as_ptr()) }
    }
}

impl Atlas {
    pub fn new(allocator: Pin<Arc<dyn Allocator>>) -> Self {
        let mut this = Self {
            inner: RefCell::new(sys::nk_font_atlas::default()),
            fonts: Vec::default(),
            allocator,
        };

        {
            let inner = this.inner.get_mut();
            inner.permanent = this.allocator.clone_inner();
            inner.temporary = this.allocator.clone_inner();
        }

        let font_ptr =
            unsafe { sys::nk_font_atlas_add_default(this.as_ptr(), 13.0, std::ptr::null_mut()) };
        this.fonts.push(Font::new(font_ptr));

        this
    }

    pub fn bake(self, format: AtlasFormat) -> Result<ImageBuilder, AtlasBakeError> {
        let mut dimensions = (0, 0);
        let raw = unsafe {
            sys::nk_font_atlas_bake(
                self.as_ptr(),
                &mut dimensions.0 as _,
                &mut dimensions.1 as _,
                format.into(),
            )
        };

        if raw.is_null() {
            return Err(AtlasBakeError::Unknown);
        }

        Ok(ImageBuilder {
            raw: raw.cast(),
            format,
            allocator: self.allocator.clone(),
            dimensions: (
                dimensions
                    .0
                    .try_into()
                    .map_err(|_| AtlasBakeError::InvalidDimensions(dimensions))?,
                dimensions
                    .1
                    .try_into()
                    .map_err(|_| AtlasBakeError::InvalidDimensions(dimensions))?,
            ),
            atlas: self,
        })
    }

    pub fn fonts(&self) -> &[Font] {
        &self.fonts
    }
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum AtlasFormat {
    Rgba32 = sys::nk_font_atlas_format_NK_FONT_ATLAS_RGBA32,
    Alpha8 = sys::nk_font_atlas_format_NK_FONT_ATLAS_ALPHA8,
}
impl AtlasFormat {
    pub fn stride(self) -> usize {
        match self {
            AtlasFormat::Rgba32 => 4,
            AtlasFormat::Alpha8 => 1,
        }
    }
}
impl Into<sys::nk_font_atlas_format> for AtlasFormat {
    fn into(self) -> sys::nk_font_atlas_format {
        self as sys::nk_font_atlas_format
    }
}

pub struct ImageBuilder {
    dimensions: (usize, usize),
    raw: *const u8,
    atlas: Atlas,
    format: AtlasFormat,
    allocator: Pin<Arc<dyn Allocator>>,
}
impl ImageBuilder {
    // TODO:  nk_draw_null_texture
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    pub fn build(self, id: usize) -> Image {
        let ImageBuilder {
            dimensions,
            raw,
            atlas,
            format,
            allocator,
        } = self;

        let atlas_ptr = atlas.as_ptr();

        let mut image = Image {
            id,
            raw,
            format,
            allocator,
            dimensions,
            atlas,
            null: sys::nk_draw_null_texture::default(),
        };

        unsafe {
            sys::nk_font_atlas_end(
                atlas_ptr,
                sys::nk_handle_id(id as i32),
                &mut image.null as _,
            )
        }

        image
    }
}

pub struct Image {
    id: usize,
    dimensions: (usize, usize),
    raw: *const u8,
    atlas: Atlas,
    format: AtlasFormat,
    allocator: Pin<Arc<dyn Allocator>>,
    null: sys::nk_draw_null_texture,
}
impl Image {
    pub fn dimensions(&self) -> (usize, usize) {
        self.dimensions
    }
    pub fn format(&self) -> AtlasFormat {
        self.format
    }
    pub fn atlas(&self) -> &Atlas {
        &self.atlas
    }
    pub fn as_slice(&self) -> &[u8] {
        let size = self.dimensions.0 * self.dimensions.1 * self.format.stride();
        unsafe { std::slice::from_raw_parts(self.raw, size) }
    }
}

#[cfg(test)]
mod tests {
    use super::{Atlas, AtlasFormat};

    #[test]
    fn attempt_bake() {
        let allocator = crate::alloc::global::create();

        let atlas = Atlas::new(allocator.clone());
        let image = atlas.bake(AtlasFormat::Rgba32).unwrap().build(1);
    }
}

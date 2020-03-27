#![deny(clippy::all, clippy::pedantic)]
#![allow(
    dead_code,
    unused_variables,
    clippy::must_use_candidate,
    clippy::missing_errors_doc
)]
#![cfg_attr(feature = "std", feature(allocator_api))]

pub mod alloc;
pub mod draw;
pub mod font;

pub use nukly_sys as sys;

use std::{cell::RefCell, pin::Pin, sync::Arc};

pub trait NuklearType<T> {
    fn as_ptr(&self) -> *mut T;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("An Unknown error occurred")]
    Unknown,
}

pub struct Nuklear {
    inner: RefCell<sys::nk_context>,
    allocator: Pin<Arc<dyn alloc::Allocator>>,
}

impl Nuklear {
    pub fn create(
        allocator: Pin<Arc<dyn alloc::Allocator>>,
        font: &font::Font,
    ) -> Result<Self, Error> {
        let inner = RefCell::new(sys::nk_context::default());

        unsafe {
            sys::nk_init(inner.as_ptr(), allocator.as_ptr(), font.handle());
        }

        Ok(Self { inner, allocator })
    }
}
/*
impl Drop for Nuklear {
    pub fn drop(self) {
        // TODO: why is this overwriting memory?
        //unsafe { sys::nk_free(self.inner.as_ptr()) }
    }
}*/

#[cfg(test)]
mod tests {
    use super::{alloc, font, Nuklear};

    #[test]
    fn full_test() {
        let allocator = alloc::global::create();

        let atlas = font::Atlas::new(allocator.clone());
        let image = atlas.bake(font::AtlasFormat::Rgba32).unwrap().build(1);
        let context = Nuklear::create(allocator, &image.atlas().fonts()[0]).unwrap();
    }
}

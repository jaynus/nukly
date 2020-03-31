const ALIGNMENT: usize = 16;

use nukly_sys as sys;
use std::ffi;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("An Unknown error occured")]
    Unknown,
    #[error("An invalid address was provided to the allocator")]
    InvalidAddress,
    #[error("An invalid size was provided to the allocator")]
    InvalidSize,
}

pub trait Allocator: crate::NuklearType<sys::nk_allocator> {
    unsafe fn alloc(&self, size: u64) -> Result<*mut ffi::c_void, Error>;
    unsafe fn dealloc(&self, ptr: *mut ffi::c_void) -> Result<(), Error>;
    fn clone_inner(&self) -> sys::nk_allocator;
}

#[allow(clippy::identity_conversion)] // This is needed to work in WASM too, as they are 32-bit pointers in wasm
#[no_mangle]
pub unsafe extern "C" fn __nukly_alloc_proxy(
    userdata: sys::nk_handle,
    old: *mut ffi::c_void,
    size: sys::nk_size,
) -> *mut ffi::c_void {
    std::panic::catch_unwind(|| {
        let this = userdata.ptr as *mut global::Allocator;
        if this.is_null() {
            return std::ptr::null_mut();
        }

        (&*this).alloc(size as u64).unwrap()
    })
    .unwrap_or_else(|e| {
        println!("nukly: Allocation failed '{:?}', {}", old, size);
        println!("Error: {:?}", e);

        std::ptr::null_mut()
    })
}

#[no_mangle]
pub unsafe extern "C" fn __nukly_free_proxy(userdata: sys::nk_handle, old: *mut ffi::c_void) {
    std::panic::catch_unwind(|| {
        let this = userdata.ptr as *mut global::Allocator;
        if this.is_null() {
            return;
        }

        let this = &mut *this;
        this.dealloc(old).unwrap()
    })
    .unwrap_or_else(|e| {
        println!("nukly: Free failed '{:?}'", old);
        println!("Error: {:?}", e);
    })
}

#[cfg(feature = "std")]
pub mod global {
    use super::ALIGNMENT;
    use nukly_sys as sys;

    use std::{
        alloc::{AllocRef, Global, Layout},
        cell::RefCell,
        convert::TryInto,
        ffi,
        pin::Pin,
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        },
    };

    #[cfg(feature = "alloc-counters")]
    #[derive(Default)]
    pub struct Counters {
        pub alloc_count: AtomicUsize,
        pub free_count: AtomicUsize,
        pub total_allocated_bytes: AtomicUsize,
        pub current_allocated_bytes: AtomicUsize,
    }

    #[inline]
    pub fn create() -> Pin<Arc<Allocator>> {
        Allocator::new()
    }

    pub struct Allocator {
        pub(crate) inner: RefCell<sys::nk_allocator>,
        #[cfg(feature = "alloc-counters")]
        counters: Counters,
    }
    impl Allocator {
        pub fn new() -> Pin<Arc<Self>> {
            let this = Arc::pin(Self {
                #[cfg(feature = "alloc-counters")]
                counters: Counters::default(),
                inner: RefCell::new(sys::nk_allocator {
                    userdata: sys::nk_handle::default(),
                    alloc: Some(super::__nukly_alloc_proxy),
                    free: Some(super::__nukly_free_proxy),
                }),
            });

            this.set_inner_ptr();

            this
        }

        // This is safe because we only use it once to set our inner value AFTER pinning
        #[allow(clippy::cast_ref_to_mut)] // TODO: fix this later
        fn set_inner_ptr(&self) {
            unsafe {
                let ref_mut: &mut Allocator = &mut *(self as *const Allocator as *mut Allocator);
                let ptr_mut: *mut Allocator = ref_mut as *mut _;
                ref_mut.inner.borrow_mut().userdata.ptr = ptr_mut as _;
            }
        }
    }
    impl crate::NuklearType<sys::nk_allocator> for Allocator {
        fn as_ptr(&self) -> *mut sys::nk_allocator {
            self.inner.as_ptr()
        }
    }
    impl super::Allocator for Allocator {
        fn clone_inner(&self) -> sys::nk_allocator {
            *self.inner.borrow()
        }
        unsafe fn alloc(&self, size: u64) -> Result<*mut ffi::c_void, super::Error> {
            let size_size = std::mem::size_of::<sys::nk_size>();
            let size = size + size_size as u64;

            let (memory, allocated_size) = Global
                .alloc(
                    Layout::from_size_align(
                        size.try_into().map_err(|_| super::Error::InvalidSize)?,
                        ALIGNMENT,
                    )
                    .map_err(|_| super::Error::InvalidSize)?,
                )
                .map_err(|_| super::Error::InvalidSize)?;

            #[allow(clippy::cast_ptr_alignment)]
            {
                *(memory.as_ptr() as *mut sys::nk_size) = allocated_size
                    .try_into()
                    .map_err(|_| super::Error::InvalidSize)?;
            }

            #[cfg(feature = "alloc-counters")]
            {
                self.counters.alloc_count.fetch_add(1, Ordering::Relaxed);
                self.counters.current_allocated_bytes.fetch_add(
                    allocated_size
                        .try_into()
                        .map_err(|_| super::Error::InvalidSize)?,
                    Ordering::Relaxed,
                );
                self.counters.total_allocated_bytes.fetch_add(
                    allocated_size
                        .try_into()
                        .map_err(|_| super::Error::InvalidSize)?,
                    Ordering::Relaxed,
                );
            }

            Ok(memory.as_ptr().add(size_size) as *mut ffi::c_void)
        }
        unsafe fn dealloc(&self, ptr: *mut ffi::c_void) -> Result<(), super::Error> {
            if ptr.is_null() {
                return Err(super::Error::InvalidAddress);
            }

            let ptr = (ptr as *mut u8).sub(std::mem::size_of::<sys::nk_size>());
            #[allow(clippy::cast_ptr_alignment)]
            let size = { *(ptr as *const sys::nk_size) };

            Global.dealloc(
                std::ptr::NonNull::new(ptr).ok_or(super::Error::InvalidAddress)?,
                Layout::from_size_align(
                    size.try_into().map_err(|e| super::Error::InvalidSize)?,
                    ALIGNMENT,
                )
                .map_err(|_| super::Error::InvalidAddress)?,
            );

            #[cfg(feature = "alloc-counters")]
            {
                self.counters.free_count.fetch_add(1, Ordering::Relaxed);
                self.counters.current_allocated_bytes.fetch_sub(
                    size.try_into().map_err(|e| super::Error::InvalidSize)?,
                    Ordering::Relaxed,
                );
            }

            Ok(())
        }
    }

    unsafe impl Send for Allocator {}
    unsafe impl Sync for Allocator {}

    #[cfg(test)]
    mod tests {
        use super::Allocator;
        use crate::alloc::{__nukly_alloc_proxy, __nukly_free_proxy};

        #[test]
        fn allocator() {
            use std::sync::atomic::Ordering;

            let allocator = Allocator::new();

            let buffer = unsafe {
                __nukly_alloc_proxy(
                    allocator.inner.borrow().userdata,
                    std::ptr::null_mut(),
                    1024,
                )
            };
            assert!(!buffer.is_null());
            assert_eq!(allocator.counters.alloc_count.load(Ordering::Relaxed), 1);
            assert_eq!(
                allocator
                    .counters
                    .total_allocated_bytes
                    .load(Ordering::Relaxed),
                1032
            );
            assert_eq!(
                allocator
                    .counters
                    .current_allocated_bytes
                    .load(Ordering::Relaxed),
                1032
            );

            unsafe { __nukly_free_proxy(allocator.inner.borrow().userdata, buffer) };
            assert_eq!(allocator.counters.free_count.load(Ordering::Relaxed), 1);
            assert_eq!(
                allocator
                    .counters
                    .current_allocated_bytes
                    .load(Ordering::Relaxed),
                0
            );
        }
    }
}

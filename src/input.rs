use crate::{sys, Nuklear, NuklearScope};

#[repr(u32)]
pub enum Button {
    Left = 0,
    Middle = 1,
    Right = 2,
    Double = 3,
}

pub trait InputScope {}
pub struct Scope;
impl InputScope for Scope {}

impl Nuklear {
    pub fn begin_input(&mut self) {
        unsafe {
            sys::nk_input_begin(self.inner.as_ptr());
        }
    }

    pub fn end_input(&mut self) {
        unsafe {
            sys::nk_input_end(self.inner.as_ptr());
        }
    }
}
/*
impl<T> NuklearScope<T>
where
    T: InputScope,
{*/
impl Nuklear {
    pub fn input_motion(&mut self, x: i32, y: i32) {
        unsafe {
            sys::nk_input_motion(self.inner.as_ptr(), x, y);
        }
    }

    pub fn input_button(&mut self, x: i32, y: i32, button: Button, state: bool) {
        unsafe {
            sys::nk_input_button(self.inner.as_ptr(), button as u32, x, y, state.into());
        }
    }
}

use crate::{sys, Error, Nuklear};
use std::ffi::CString;

const NK_VERTEX_LAYOUT_END: sys::nk_draw_vertex_layout_element =
    sys::nk_draw_vertex_layout_element {
        attribute: sys::nk_draw_vertex_layout_attribute_NK_VERTEX_ATTRIBUTE_COUNT,
        format: sys::nk_draw_vertex_layout_format_NK_FORMAT_COUNT,
        offset: 0,
    };

#[repr(C)]
pub struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
    color: [u8; 4],
}
impl Vertex {
    const LAYOUT: [sys::nk_draw_vertex_layout_element; 4] = [
        sys::nk_draw_vertex_layout_element {
            attribute: sys::nk_draw_vertex_layout_attribute_NK_VERTEX_POSITION,
            format: sys::nk_draw_vertex_layout_format_NK_FORMAT_FLOAT,
            offset: 0,
        },
        sys::nk_draw_vertex_layout_element {
            attribute: sys::nk_draw_vertex_layout_attribute_NK_VERTEX_TEXCOORD,
            format: sys::nk_draw_vertex_layout_format_NK_FORMAT_FLOAT,
            offset: std::mem::size_of::<f32>() * 2,
        },
        sys::nk_draw_vertex_layout_element {
            attribute: sys::nk_draw_vertex_layout_attribute_NK_VERTEX_COLOR,
            format: sys::nk_draw_vertex_layout_format_NK_FORMAT_R8G8B8A8,
            offset: std::mem::size_of::<f32>() * 4,
        },
        NK_VERTEX_LAYOUT_END,
    ];

    pub fn apply(config: &mut sys::nk_convert_config) {
        config.vertex_layout = Self::LAYOUT.as_ptr();
        config.vertex_size = std::mem::size_of::<Self>() as usize;
        config.vertex_alignment = std::mem::align_of::<Self>() as usize;
    }
}

pub struct PrepareDrawHandle(sys::nk_buffer);

impl Nuklear {
    const DEFAULT_BUFFER_INITIAL_SIZE: usize = 4 * 1024;

    pub fn prepare_draw(
        &mut self,
        vertices: &mut [u8],
        elements: &mut [u8],
    ) -> Result<PrepareDrawHandle, Error> {
        let mut vertex_buf: sys::nk_buffer = sys::nk_buffer::default();
        let mut element_buf: sys::nk_buffer = sys::nk_buffer::default();

        let mut commands: sys::nk_buffer = sys::nk_buffer::default();

        unsafe {
            sys::nk_buffer_init(
                &mut commands as _,
                self.allocator.as_ptr(),
                Self::DEFAULT_BUFFER_INITIAL_SIZE,
            );

            sys::nk_buffer_init_fixed(
                &mut vertex_buf as _,
                vertices.as_mut_ptr() as _,
                vertices.len() as usize,
            );
            sys::nk_buffer_init_fixed(
                &mut element_buf as _,
                elements.as_mut_ptr() as _,
                elements.len() as usize,
            );

            sys::nk_convert(
                self.inner.as_ptr(),
                &mut commands as _,
                &mut vertex_buf as _,
                &mut element_buf as _,
                &mut self.vertex_config as _,
            );
        }

        Ok(PrepareDrawHandle(commands))
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn draw<F>(&mut self, handle: PrepareDrawHandle, mut each_command: F) -> Result<(), Error>
    where
        F: FnMut(&mut Self, &sys::nk_draw_command),
    {
        let mut commands = handle.0;

        let mut draw_command =
            unsafe { sys::nk__draw_begin(self.inner.as_ptr(), &mut commands as _) };

        while !draw_command.is_null() {
            let command = unsafe { &*draw_command };

            if command.elem_count > 0 {
                (each_command)(self, command);
            }

            draw_command = unsafe {
                sys::nk__draw_next(draw_command, &mut commands as _, self.inner.as_ptr())
            };
        }

        unsafe { sys::nk_clear(self.inner.as_ptr()) };

        Ok(())
    }
}

impl Nuklear {
    pub fn begin<F, S>(
        &self,
        title: S,
        dimensions: (f32, f32, f32, f32),
        flags: PanelFlags,
        mut f: F,
    ) where
        S: AsRef<str>,
        F: FnMut(&Self),
    {
        unsafe {
            let title_str = CString::new(title.as_ref()).unwrap();
            if sys::nk_begin(
                self.inner.as_ptr(),
                title_str.as_ptr(),
                sys::nk_rect::from(dimensions),
                flags.bits,
            ) != 0
            {
                (f)(self)
            }
            sys::nk_end(self.inner.as_ptr());
        }
    }

    pub fn button_label<S>(&self, label: S) -> bool
    where
        S: AsRef<str>,
    {
        let label = CString::new(label.as_ref()).unwrap();
        unsafe { sys::nk_button_label(self.inner.as_ptr(), label.as_ptr()) != 0 }
    }

    pub fn layout_row_static(&self, height: f32, width: f32, count: usize) {
        unsafe {
            sys::nk_layout_row_static(
                self.inner.as_ptr(),
                height,
                width.round() as i32,
                count as i32,
            )
        }
    }

    pub fn layout_row_dynamic(&self, height: f32, cols: usize) {
        unsafe { sys::nk_layout_row_dynamic(self.inner.as_ptr(), height, cols as i32) }
    }

    pub fn layout_row_begin(&self, format: sys::nk_layout_format, height: f32, cols: usize) {
        unsafe { sys::nk_layout_row_begin(self.inner.as_ptr(), format, height, cols as i32) }
    }
}

bitflags::bitflags! {
    pub struct WindowFlags: sys::nk_window_flags {
        const PRIVATE = sys::nk_window_flags_NK_WINDOW_PRIVATE;
        const DYNAMIC = sys::nk_window_flags_NK_WINDOW_DYNAMIC;
        const ROM = sys::nk_window_flags_NK_WINDOW_ROM;
        const NOT_INTERACTIVE = sys::nk_window_flags_NK_WINDOW_NOT_INTERACTIVE;
        const HIDDEN = sys::nk_window_flags_NK_WINDOW_HIDDEN;
        const CLOSED = sys::nk_window_flags_NK_WINDOW_CLOSED ;
        const MINIMIZED = sys::nk_window_flags_NK_WINDOW_MINIMIZED;
        const REMOVE_ROM = sys::nk_window_flags_NK_WINDOW_REMOVE_ROM;
    }
}

bitflags::bitflags! {
    pub struct PanelFlags: sys::nk_panel_flags {
        const BORDER = sys::nk_panel_flags_NK_WINDOW_BORDER;
        const MOVABLE = sys::nk_panel_flags_NK_WINDOW_MOVABLE;
        const SCALABLE = sys::nk_panel_flags_NK_WINDOW_SCALABLE;
        const CLOSABLE = sys::nk_panel_flags_NK_WINDOW_CLOSABLE;
        const MINIMIZABLE = sys::nk_panel_flags_NK_WINDOW_MINIMIZABLE;
        const NO_SCROLLBAR = sys::nk_panel_flags_NK_WINDOW_NO_SCROLLBAR;
        const TITLE = sys::nk_panel_flags_NK_WINDOW_TITLE;
        const SCROLL_AUTO_HIDE = sys::nk_panel_flags_NK_WINDOW_SCROLL_AUTO_HIDE;
        const BACKGROUND = sys::nk_panel_flags_NK_WINDOW_BACKGROUND;
        const SCALE_LEFT = sys::nk_panel_flags_NK_WINDOW_SCALE_LEFT;
        const NO_INPUT = sys::nk_panel_flags_NK_WINDOW_NO_INPUT;
    }
}

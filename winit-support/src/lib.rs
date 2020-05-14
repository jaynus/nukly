use nukly::{input::Button, input::Scope as Input, Nuklear};
use std::marker::PhantomData;
use winit::{
    dpi::LogicalPosition,
    event::{DeviceEvent, ElementState, Event, MouseButton, WindowEvent},
    window::Window,
};

#[inline(always)]
pub fn winit_to_nk_button(button: MouseButton) -> Button {
    match button {
        MouseButton::Left => Button::Left,
        MouseButton::Right => Button::Left,
        MouseButton::Middle => Button::Left,
        _ => unimplemented!(),
    }
}

pub struct NuklyWindowEventHandler {
    last_mouse_position: LogicalPosition<f64>,
}
impl Default for NuklyWindowEventHandler {
    fn default() -> Self {
        Self {
            last_mouse_position: LogicalPosition { x: 0.0, y: 0.0 },
        }
    }
}
impl NuklyWindowEventHandler {
    pub fn handle_event<T>(&mut self, context: &mut Nuklear, event: &Event<T>) {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CursorMoved { position, .. } => {
                    self.last_mouse_position = position.to_logical(1.0);
                    context.input_motion(
                        self.last_mouse_position.x as i32,
                        self.last_mouse_position.y as i32,
                    );
                }
                WindowEvent::MouseInput { state, button, .. } => context.input_button(
                    self.last_mouse_position.x as i32,
                    self.last_mouse_position.y as i32,
                    winit_to_nk_button(*button),
                    *state == ElementState::Pressed,
                ),
                _ => {}
            },
            _ => {} // ignore
        }
    }

    pub fn finish(&mut self, context: &mut Nuklear) {}
}

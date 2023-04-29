use std::collections::HashMap;

use macroquad::{miniquad::EventHandler, prelude::*};
const HOLD_DURATION: f32 = 0.3;
const CLICK_SPEED: f32 = 0.2;

#[derive(Default)]
struct MouseButtonDriver {
    successive_clicks: u32,
    held: bool,
    /// If [held] is true, this refers to how long the button has been held. If [held] is false, this
    /// refers to how long the button has been released
    duration: f32,
    hold_started_at: Vec2,
}

impl MouseButtonDriver {
    fn update(&mut self) {
        let frame_time = get_frame_time();
        if self.duration > CLICK_SPEED {
            self.successive_clicks = 0;
        }
        self.duration += frame_time;

        log::trace!("duration: {}", self.duration);
        log::trace!("held: {}", self.held);
    }

    fn held(&self) -> Option<(Vec2, f32)> {
        (self.held && self.duration >= HOLD_DURATION)
            .then_some((self.hold_started_at, self.duration))
    }

    fn double_clicked(&self) -> bool {
        self.successive_clicks == 2 && self.held
    }

    fn listen_event(&mut self, pressed: bool, x: f32, y: f32) {
        self.held = pressed;
        if pressed {
            self.successive_clicks += 1;
        }
        self.hold_started_at = vec2(x, y);
        self.duration = 0.0;
    }
}

/// Extra input driver on top of macroquad, detecting more events such as double clicks. Must be
/// manually updated through the loop.
pub struct InputDriver {
    subscribe_id: usize,
    mouse_buttons: HashMap<MouseButton, MouseButtonDriver>,
    mouse_position: [Vec2; 2],
}

impl Default for InputDriver {
    fn default() -> Self {
        Self {
            subscribe_id: macroquad::input::utils::register_input_subscriber(),
            mouse_buttons: [MouseButton::Left, MouseButton::Right, MouseButton::Middle]
                .into_iter()
                .map(|button| (button, MouseButtonDriver::default()))
                .collect(),
            mouse_position: [Vec2::ZERO; 2],
        }
    }
}

impl EventHandler for InputDriver {
    fn update(&mut self, _ctx: &mut miniquad::Context) {}
    fn draw(&mut self, _ctx: &mut miniquad::Context) {}

    fn mouse_button_down_event(
        &mut self,
        _: &mut miniquad::Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) {
        if let Some(button) = self.mouse_buttons.get_mut(&button) {
            button.listen_event(true, x, y)
        }
    }

    fn mouse_button_up_event(
        &mut self,
        _: &mut miniquad::Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) {
        if let Some(button) = self.mouse_buttons.get_mut(&button) {
            button.listen_event(false, x, y)
        }
    }
}

impl InputDriver {
    pub fn update(&mut self) {
        macroquad::input::utils::repeat_all_miniquad_input(self, self.subscribe_id);
        self.mouse_buttons.values_mut().for_each(|b| b.update());

        self.mouse_position.rotate_left(1);
        self.mouse_position[1] = mouse_position().into();
    }
}

#[allow(dead_code)]
impl InputDriver {
    pub fn rmb_doubleclick(&self) -> bool {
        self.mouse_buttons[&MouseButton::Right].double_clicked()
    }

    pub fn lmb_doubleclick(&self) -> bool {
        self.mouse_buttons[&MouseButton::Left].double_clicked()
    }

    pub fn rmb_hold(&self) -> Option<(Vec2, f32)> {
        self.mouse_buttons[&MouseButton::Right].held()
    }

    pub fn lmb_hold(&self) -> Option<(Vec2, f32)> {
        self.mouse_buttons[&MouseButton::Left].held()
    }

    pub fn mouse_delta(&self) -> Vec2{
        self.mouse_position[1] - self.mouse_position[0]
    }
}

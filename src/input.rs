use macroquad::prelude::*;

use crate::mouse;

const HOLD_DURATION: f32 = 0.2;
const CLICK_SPEED: f32 = 0.25;

struct MouseButtonDriver {
    button: MouseButton,
    successive_clicks: u32,
    held: bool,
    /// If [held] is true, this refers to how long the button has been held. If [held] is false, this
    /// refers to how long the button has been released
    duration: f32,
    hold_started_at: Vec2,
}

impl MouseButtonDriver {
    fn button(button: MouseButton) -> Self {
        Self {
            button,
            successive_clicks: 0,
            held: false,
            duration: 0.0,
            hold_started_at: Vec2::ZERO,
        }
    }

    fn update(&mut self) {
        let frame_time = get_frame_time();

        // mouse button released
        if self.held && !is_mouse_button_down(self.button) {
            self.held = false;
            if self.duration > CLICK_SPEED {
                self.successive_clicks = 0;
            }
            self.duration = 0.0;
        }
        // mouse button pressed
        else if !self.held && is_mouse_button_down(self.button) {
            self.held = true;
            self.hold_started_at = mouse();
            if self.duration > CLICK_SPEED {
                self.successive_clicks = 0;
            }
            self.successive_clicks += 1;

            self.duration = 0.0;
        }

        self.duration += frame_time;
    }

    pub fn held(&self) -> Option<(Vec2, f32)> {
        (self.held && self.duration >= HOLD_DURATION)
            .then_some((self.hold_started_at, self.duration))
    }

    pub fn double_clicked(&self) -> bool {
        self.successive_clicks == 2 && self.held
    }
}

/// Extra input driver on top of macroquad, detecting more events such as double clicks. Must be
/// manually updated through the loop.
pub struct InputDriver {
    right_mouse_button: MouseButtonDriver,
    left_mouse_button: MouseButtonDriver,
}

impl Default for InputDriver {
    fn default() -> Self {
        Self {
            right_mouse_button: MouseButtonDriver::button(MouseButton::Right),
            left_mouse_button: MouseButtonDriver::button(MouseButton::Left),
        }
    }
}

impl InputDriver {
    pub fn update(&mut self) {
        self.right_mouse_button.update();
        self.left_mouse_button.update();
    }

    pub fn rmb_doubleclick(&self) -> bool {
        self.right_mouse_button.double_clicked()
    }

    pub fn lmb_doubleclick(&self) -> bool {
        self.left_mouse_button.double_clicked()
    }

    pub fn rmb_hold(&self) -> Option<(Vec2, f32)> {
        self.right_mouse_button.held()
    }

    pub fn lmb_hold(&self) -> Option<(Vec2, f32)> {
        self.left_mouse_button.held()
    }
}

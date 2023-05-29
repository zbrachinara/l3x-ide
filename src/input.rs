use std::collections::HashMap;

use egui::Context;
use macroquad::{miniquad::EventHandler, prelude::*};
const HOLD_DURATION: f32 = 0.3;
const CLICK_SPEED: f32 = 0.2;

#[derive(PartialEq, Default, Clone, Copy)]
enum HoldState {
    #[default]
    None,
    JustSeen,
    Holding,
}

impl HoldState {
    fn update(self, press_state: bool) -> Self {
        if press_state {
            match self {
                Self::None => Self::JustSeen,
                Self::JustSeen | Self::Holding => Self::Holding,
            }
        } else {
            Self::None
        }
    }

    fn holding(&self) -> bool {
        matches!(self, Self::JustSeen | Self::Holding)
    }
}

#[derive(Default)]
pub struct MouseButtonDriver {
    successive_clicks: u32,
    pressed: bool,
    /// If [held] is true, this refers to how long the button has been held. If [held] is false, this
    /// refers to how long the button has been released
    duration: f32,
    hold_started_at: Vec2,
    press_started_this_frame: bool,
    hold_state: HoldState,
}

/// Private functions which facilitate updating the driver's internal state
impl MouseButtonDriver {
    fn update(&mut self, over_ui: bool) {
        let frame_time = get_frame_time();
        if self.duration > CLICK_SPEED {
            self.successive_clicks = 0;
        }
        if self.press_started_this_frame && over_ui {
            self.successive_clicks = 0;
            self.duration = 0.;
            self.pressed = false;
        }
        self.duration += frame_time;

        log::trace!("duration: {}", self.duration);
        log::trace!("held: {}", self.pressed);
        self.press_started_this_frame = false;
        self.hold_state = self.hold_state.update(self.satsifies_hold())
    }

    fn listen_event(&mut self, pressed: bool, x: f32, y: f32) {
        self.pressed = pressed;
        if pressed {
            self.successive_clicks += 1;
            self.press_started_this_frame = true;
        }
        self.hold_started_at = vec2(x, y);
        self.duration = 0.0;
    }

    fn satsifies_hold(&self) -> bool {
        self.pressed && self.duration >= HOLD_DURATION
    }
}

impl MouseButtonDriver {
    pub fn started_holding(&self) -> Option<Vec2> {
        (self.hold_state == HoldState::JustSeen).then_some(self.hold_started_at)
    }

    pub fn held(&self) -> Option<(Vec2, f32)> {
        self.hold_state
            .holding()
            .then_some((self.hold_started_at, self.duration))
    }

    pub fn double_clicked(&self) -> bool {
        self.successive_clicks == 2 && self.pressed
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
    pub fn update(&mut self, egui_ctx: &Context) {
        macroquad::input::utils::repeat_all_miniquad_input(self, self.subscribe_id);
        self.mouse_buttons
            .values_mut()
            .for_each(|b| b.update(egui_ctx.is_pointer_over_area()));

        self.mouse_position.rotate_left(1);
        self.mouse_position[1] = mouse_position().into();
    }
}

#[allow(dead_code)]
impl InputDriver {
    pub fn rmb(&self) -> &MouseButtonDriver {
        &self.mouse_buttons[&MouseButton::Right]
    }

    pub fn lmb(&self) -> &MouseButtonDriver {
        &self.mouse_buttons[&MouseButton::Left]
    }

    pub fn mouse_delta(&self) -> Vec2 {
        self.mouse_position[1] - self.mouse_position[0]
    }
}

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
        println!("duration: {}", self.duration);
        println!("held: {}", self.held);
    }

    pub fn held(&self) -> Option<(Vec2, f32)> {
        (self.held && self.duration >= HOLD_DURATION)
            .then_some((self.hold_started_at, self.duration))
    }

    pub fn double_clicked(&self) -> bool {
        self.successive_clicks == 2 && self.held
    }

    fn listen_event(&mut self, pressed: bool, x: f32, y: f32) {
        self.held = pressed;
        if pressed {
            self.successive_clicks += 1;
        }
        self.hold_started_at = Vec2 { x, y };
        self.duration = 0.0;
    }
}

/// Extra input driver on top of macroquad, detecting more events such as double clicks. Must be
/// manually updated through the loop.
pub struct InputDriver {
    subscribe_id: usize,
    right_mouse_button: MouseButtonDriver,
    left_mouse_button: MouseButtonDriver,
}

impl Default for InputDriver {
    fn default() -> Self {
        Self {
            subscribe_id: macroquad::input::utils::register_input_subscriber(),
            right_mouse_button: MouseButtonDriver::default(),
            left_mouse_button: MouseButtonDriver::default(),
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
        match button {
            MouseButton::Left => &mut self.left_mouse_button,
            MouseButton::Right => &mut self.right_mouse_button,
            _ => return,
        }
        .listen_event(true, x, y)
    }

    fn mouse_button_up_event(
        &mut self,
        _: &mut miniquad::Context,
        _button: MouseButton,
        x: f32,
        y: f32,
    ) {
        match _button {
            MouseButton::Left => &mut self.left_mouse_button,
            MouseButton::Right => &mut self.right_mouse_button,
            _ => return,
        }
        .listen_event(false, x, y)
    }
}

impl InputDriver {
    pub fn update(&mut self) {
        macroquad::input::utils::repeat_all_miniquad_input(self, self.subscribe_id);
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

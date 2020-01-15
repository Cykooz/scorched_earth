//! Typedefs for input shortcuts.
use ggez::event::KeyCode;
use ggez_goodies::input;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Button {
    Up,
    Down,
    Left,
    Right,
    Select,
    Quit,
    Fire,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Axis {
    Vertical,
    Horizontal,
}

pub type Binding = input::InputBinding<Axis, Button>;
pub type Event = input::InputEffect<Axis, Button>;
pub type State = input::InputState<Axis, Button>;

/// Create the default keybindings for our input state.
pub fn create_input_binding() -> Binding {
    Binding::new()
        .bind_key_to_button(KeyCode::Up, Button::Up)
        .bind_key_to_button(KeyCode::Down, Button::Down)
        .bind_key_to_button(KeyCode::Left, Button::Left)
        .bind_key_to_button(KeyCode::Right, Button::Right)
        .bind_key_to_button(KeyCode::Return, Button::Select)
        .bind_key_to_button(KeyCode::Escape, Button::Quit)
        .bind_key_to_button(KeyCode::Space, Button::Fire)
}

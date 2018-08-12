use maths::Vector2i;
use sdl2;
pub use sdl2::{keyboard::Keycode, mouse::MouseButton};
use std::{collections::HashMap, error, fmt};

///Errors related to input management.
#[derive(Debug)]
pub enum InputError {
    KeycodeNotFound(Keycode),
    MouseButtonNotFound(MouseButton),
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InputError::KeycodeNotFound(keycode) => write!(f, "Keycode not found: {}", keycode),
            InputError::MouseButtonNotFound(button) => {
                write!(f, "MouseButton not found: {:?}", button)
            }
        }
    }
}

impl error::Error for InputError {}

///Represents the current state of a keyboard key or mouse button.
pub struct KeyState {
    down: bool,
    changed: bool,
}

impl KeyState {
    ///Is the key currently held down?
    pub fn down(&self) -> bool {
        self.down
    }

    ///Is the key currently up?
    pub fn up(&self) -> bool {
        !self.down
    }

    ///Did the key go from up to down this frame?
    pub fn pressed(&self) -> bool {
        self.down && self.changed
    }

    ///Did the key go from down to up this frame?
    pub fn released(&self) -> bool {
        !self.down && self.changed
    }

    ///Update the keystate with a new state.
    fn update(&mut self, pressed: bool) {
        self.changed = pressed != self.down;
        self.down = pressed;
    }
}

///Retrieves and manages input from events.
pub struct InputManager {
    //Keyboard state
    key_state: HashMap<Keycode, KeyState>,
    //Mouse state
    mouse_state: HashMap<MouseButton, KeyState>,
    mouse_position: Vector2i,
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}

impl InputManager {
    ///Initializes a new InputManager.
    pub fn new() -> InputManager {
        InputManager {
            key_state: HashMap::new(),
            mouse_state: HashMap::new(),
            mouse_position: Vector2i::new(0, 0),
        }
    }

    ///Update InputManager with new events from the event pump.
    pub fn update(&mut self, events: &sdl2::EventPump) {
        //Update keyboard state
        let new_key_state = events.keyboard_state();
        let key_states = new_key_state.scancodes().filter_map(|(scancode, pressed)| {
            if let Some(keycode) = Keycode::from_scancode(scancode) {
                Some((keycode, pressed))
            } else {
                None
            }
        });

        for (keycode, pressed) in key_states {
            self.key_state
                .entry(keycode)
                .or_insert(KeyState {
                    down: pressed,
                    changed: false,
                })
                .update(pressed);
        }

        //Update mouse
        let mouse_state = events.mouse_state();
        self.mouse_position = Vector2i::new(mouse_state.x(), mouse_state.y());

        for (button, pressed) in mouse_state.mouse_buttons() {
            self.mouse_state
                .entry(button)
                .or_insert(KeyState {
                    down: pressed,
                    changed: false,
                })
                .update(pressed);
        }
    }

    ///Get the current state of a keyboard key.
    pub fn key(&self, keycode: Keycode) -> Result<&KeyState, InputError> {
        self.key_state
            .get(&keycode)
            .ok_or_else(|| InputError::KeycodeNotFound(keycode))
    }

    ///Get the current state of a mouse button.
    pub fn button(&self, button: MouseButton) -> Result<&KeyState, InputError> {
        self.mouse_state
            .get(&button)
            .ok_or_else(|| InputError::MouseButtonNotFound(button))
    }

    ///Mouse position in pixels, relative to the top left corner.
    pub fn mouse_position(&self) -> Vector2i {
        self.mouse_position
    }
}

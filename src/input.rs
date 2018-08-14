use maths::Vector2i;
use sdl2;
pub use sdl2::{keyboard::Keycode, mouse::MouseButton};
use std::{collections::HashMap, error, fmt};

/// Errors related to input management.
#[derive(Debug)]
pub enum InputError {
    KeycodeNotFound(Keycode),
    KeybindNotFound(String),
    MouseButtonNotFound(MouseButton),
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InputError::KeycodeNotFound(keycode) => write!(f, "Keycode not found: {}", keycode),
            InputError::KeybindNotFound(keybind) => write!(f, "Keybind not found: {}", keybind),
            InputError::MouseButtonNotFound(button) => {
                write!(f, "MouseButton not found: {:?}", button)
            }
        }
    }
}

impl error::Error for InputError {}

/// Represents the current state of a keyboard key or mouse button.
pub struct KeyState {
    down: bool,
    changed: bool,
}

impl KeyState {
    /// Is the key currently held down?
    pub fn down(&self) -> bool {
        self.down
    }

    /// Is the key currently up?
    pub fn up(&self) -> bool {
        !self.down
    }

    /// Did the key go from up to down this frame?
    pub fn pressed(&self) -> bool {
        self.down && self.changed
    }

    /// Did the key go from down to up this frame?
    pub fn released(&self) -> bool {
        !self.down && self.changed
    }

    /// Update the keystate with a new state.
    fn update(&mut self, pressed: bool) {
        self.changed = pressed != self.down;
        self.down = pressed;
    }
}

/// Retrieves and manages input from events.
pub struct InputManager {
    //Keyboard state
    key_state: HashMap<Keycode, KeyState>,
    //Keybinds
    keybinds: HashMap<String, Keycode>,
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
    /// Initializes a new InputManager.
    pub fn new() -> InputManager {
        InputManager {
            key_state: HashMap::new(),
            keybinds: HashMap::new(),
            mouse_state: HashMap::new(),
            mouse_position: Vector2i::new(0, 0),
        }
    }

    /// Updates InputManager with new events from an event pump.
    ///
    /// This should be called every frame for the `InputManager`
    /// to work properly.
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

    /// Gets the current state of a keyboard key.
    ///
    /// Returns [`KeycodeNotFound`](enum.InputError.html#variant.KeycodeNotFound)
    /// if the keycode is not found in the current keyboard state.
    ///
    /// # Example
    ///
    /// ```
    /// if input_manager.key(Keycode::Space)?.pressed() {
    ///     println!("Space pressed!");
    /// }
    /// ```
    pub fn key(&self, keycode: Keycode) -> Result<&KeyState, InputError> {
        self.key_state
            .get(&keycode)
            .ok_or_else(|| InputError::KeycodeNotFound(keycode))
    }

    /// Gets the current state of a custom keybind.
    ///
    /// Returns [`KeybindNotFound`](enum.InputError.html#variant.KeybindNotFound)
    /// if the keybind name is not set.
    ///
    /// Can also return the same error(s) as [`key`](#method.key).
    ///
    /// # Example
    ///
    /// ```
    /// if input_manager.keybind("Space")?.pressed() {
    ///     println!("Space pressed!");
    /// }
    /// ```
    pub fn keybind(&self, name: &str) -> Result<&KeyState, InputError> {
        match self.keybinds.get(name) {
            None => Err(InputError::KeybindNotFound(name.to_owned())),
            Some(&keycode) => self.key(keycode),
        }
    }

    /// Sets a custom keybind for chosen [`Keycode`](enum.Keycode.html).
    ///
    /// # Example
    ///
    /// ```
    /// input_manager.set_keybind("Space", Keycode::Space);
    /// ```
    pub fn set_keybind(&mut self, name: &str, keycode: Keycode) {
        self.keybinds.insert(name.to_owned(), keycode);
    }

    /// Removes a keybind.
    ///
    /// # Example
    ///
    /// ```
    /// input_manager.clear_keybind("Space");
    /// ```
    pub fn clear_keybind(&mut self, name: &str) {
        self.keybinds.remove(name);
    }

    /// Gets the current state of a mouse button.
    ///
    /// Returns [`MouseButtonNotFound`](enum.InputError.html#variant.MouseButtonNotFound)
    /// if the button is not found in the current mouse state.
    ///
    /// # Example
    ///
    /// ```
    /// if input_manager.button(MouseButton::Right)?.released() {
    ///     println!("Right click released!");
    /// }
    /// ```
    pub fn button(&self, button: MouseButton) -> Result<&KeyState, InputError> {
        self.mouse_state
            .get(&button)
            .ok_or_else(|| InputError::MouseButtonNotFound(button))
    }

    /// Gets the current mouse position in pixels,
    /// relative to the top left corner of the window.
    pub fn mouse_position(&self) -> Vector2i {
        self.mouse_position
    }
}

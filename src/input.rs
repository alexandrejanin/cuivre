use maths::Vector2i;
use sdl2;
pub use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Keycode,
    mouse::MouseButton,
};
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
    /// Update the keystate with a new state.
    fn update(&mut self, down: bool) {
        self.changed = down != self.down;
        self.down = down;
    }

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
    mouse_position_relative: Vector2i,
    mouse_wheel: i32,
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
            mouse_position_relative: Vector2i::new(0, 0),
            mouse_wheel: 0,
        }
    }

    /// Updates InputManager with new events from an event pump.
    ///
    /// This should be called at the start of your game loop.
    ///
    /// Returns events that aren't handled by the `InputManager`.
    pub fn update(&mut self, mut events: sdl2::EventPump) -> Vec<Event> {
        self.mouse_wheel = 0;
        self.mouse_position_relative = Vector2i::new(0, 0);

        for keystate in self.key_state.values_mut() {
            let down = keystate.down;
            keystate.update(down)
        }

        // List of events that aren't handled and will be returned
        let mut passthrough_events = Vec::new();

        for event in events.poll_iter() {
            match event {
                Event::KeyDown { keycode, .. } => if let Some(keycode) = keycode {
                    self.key_state
                        .entry(keycode)
                        .or_insert(KeyState {
                            down: false,
                            changed: false,
                        }).update(true)
                },

                Event::KeyUp { keycode, .. } => if let Some(keycode) = keycode {
                    self.key_state
                        .entry(keycode)
                        .or_insert(KeyState {
                            down: true,
                            changed: false,
                        }).update(false)
                },

                Event::MouseButtonDown { mouse_btn, .. } => self
                    .mouse_state
                    .entry(mouse_btn)
                    .or_insert(KeyState {
                        down: false,
                        changed: false,
                    }).update(true),

                Event::MouseButtonUp { mouse_btn, .. } => self
                    .mouse_state
                    .entry(mouse_btn)
                    .or_insert(KeyState {
                        down: true,
                        changed: false,
                    }).update(true),

                Event::MouseWheel { y, .. } => self.mouse_wheel = y,

                Event::MouseMotion {
                    x, y, xrel, yrel, ..
                } => {
                    self.mouse_position = Vector2i::new(x, y);
                    self.mouse_position_relative = Vector2i::new(xrel, yrel);
                }

                // Other events are ignored
                _ => passthrough_events.push(event),
            }
        }

        passthrough_events
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
    pub fn key(&self, keycode: Keycode) -> &KeyState {
        match self.key_state.get(&keycode) {
            Some(keystate) => &keystate,
            None => &KeyState {
                down: false,
                changed: false,
            },
        }
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
            Some(&keycode) => Ok(self.key(keycode)),
            None => Err(InputError::KeybindNotFound(name.to_owned())),
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

    /// Gets the current mouse position in pixels,
    /// relative to last frame's mouse position.
    pub fn mouse_position_relative(&self) -> Vector2i {
        self.mouse_position_relative
    }

    /// Gets the current state of the mouse wheel.
    /// < 0: scrolling down
    /// > 0: scrolling up
    pub fn mouse_wheel(&self) -> i32 {
        self.mouse_wheel
    }
}

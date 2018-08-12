use maths::Vector2i;
use sdl2;
pub use sdl2::keyboard::Keycode;
use std::collections::HashMap;

///Represents the current state of a keyboard key.
struct KeyState {
    down: bool,
    changed: bool,
}

impl KeyState {
    ///Is the key currently held down?
    fn down(&self) -> bool {
        self.down
    }

    ///Is the key currently up?
    fn up(&self) -> bool {
        !self.down
    }

    ///Did the key go from up to down this frame?
    fn pressed(&self) -> bool {
        self.down && self.changed
    }

    ///Did the key go from down to up this frame?
    fn released(&self) -> bool {
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
            mouse_position: Vector2i::new(0, 0),
        }
    }

    ///Update InputManager with new events from the event pump.
    pub fn update(&mut self, events: &sdl2::EventPump) {
        //Update keyboard state
        let new_key_state: Vec<(Keycode, bool)> = events
            .keyboard_state()
            .scancodes()
            //Convert from scancode to keycode
            .filter_map(|(scancode, pressed)|
                if let Some(keycode) = Keycode::from_scancode(scancode) {
                    Some((keycode, pressed))
                } else {
                    None
                }
            )
            .collect();

        for (keycode, pressed) in &new_key_state {
            if !self.key_state.contains_key(keycode) {
                self.key_state.insert(
                    *keycode,
                    KeyState {
                        down: *pressed,
                        changed: false,
                    },
                );
            }

            self.key_state
                .get_mut(keycode)
                .unwrap_or_else(|| panic!("Keycode not found: {:?}", keycode))
                .update(*pressed);
        }

        //Update mouse
        let mouse_state = events.mouse_state();
        self.mouse_position = Vector2i::new(mouse_state.x(), mouse_state.y());
    }

    ///Is the key currently held down?
    pub fn key_down(&self, keycode: Keycode) -> bool {
        self.key_state[&keycode].down()
    }

    ///Is the key currently up?
    pub fn key_up(&self, keycode: Keycode) -> bool {
        self.key_state[&keycode].up()
    }

    ///Did the key go from up to down this frame?
    pub fn key_pressed(&self, keycode: Keycode) -> bool {
        self.key_state[&keycode].pressed()
    }

    ///Did the key go from down to up this frame?
    pub fn key_released(&self, keycode: Keycode) -> bool {
        self.key_state[&keycode].released()
    }

    ///Mouse position in pixels, relative to the top left corner.
    pub fn mouse_position(&self) -> Vector2i {
        self.mouse_position
    }
}

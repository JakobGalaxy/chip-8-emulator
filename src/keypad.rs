const NUM_KEYS: u32 = 16;

pub struct Keypad {
    key_states: [bool; (NUM_KEYS as usize)],
}

impl Keypad {
    pub fn new() -> Self {
        return Keypad {
            key_states: [false; (NUM_KEYS as usize)],
        };
    }

    pub fn set_key(&mut self, key_id: usize) {
        self.key_states[key_id] = true;
    }

    pub fn unset_key(&mut self, key_id: usize) {
        self.key_states[key_id] = false;
    }
}
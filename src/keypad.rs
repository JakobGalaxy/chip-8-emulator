const NUM_KEYS: u32 = 16;

#[derive(Clone)]
pub struct Keypad {
    key_states: [bool; (NUM_KEYS as usize)],
}

impl Keypad {
    pub fn new() -> Self {
        return Keypad {
            key_states: [false; (NUM_KEYS as usize)],
        };
    }

    pub fn set_key(&mut self, key_id: u8) {
        self.key_states[key_id as usize] = true;
    }

    pub fn unset_key(&mut self, key_id: u8) {
        self.key_states[key_id as usize] = false;
    }

    pub fn check_key_state(&self, key_id: u8) -> bool {
        return self.key_states[key_id as usize];
    }

    /// returns the first keypress, if available
    pub fn get_keypress(&self) -> Option<u8> {
        for (idx, key_state) in self.key_states.iter().enumerate() {
            if *key_state {
                return Some(idx as u8);
            }
        }
        return None;
    }
}
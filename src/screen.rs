use crate::SDLScreenUI;

pub const HEIGHT: usize = 32;
pub const WIDTH: usize = 64;

pub struct Screen {
    /// access pixel values using `pixel_vals[y][x]` (`x` = horizontal; `y` = vertical)
    pixel_vals: [[bool; WIDTH]; HEIGHT],

    screen_ui: SDLScreenUI,
}

impl Screen {
    pub fn new(screen_ui: SDLScreenUI) -> Screen {
        return Screen {
            pixel_vals: [[false; WIDTH]; HEIGHT],
            screen_ui,
        };
    }

    /// returns `true` if a pixel was turned off in the process (set `VF` to `1`)
    pub fn display_sprite(&mut self, x_pos: u8, y_pos: u8, sprite_data: &[u8]) -> bool {
        let x_pos = x_pos % (WIDTH as u8);
        let y_pos = y_pos % (HEIGHT as u8);

        let mut pixel_turned_off = false;

        for (byte_idx, byte) in sprite_data.iter().enumerate() {
            let curr_y = y_pos + (byte_idx as u8);

            if curr_y >= (HEIGHT as u8) {
                // this should achieve a clipping behaviour
                continue;
            }

            for bit_idx in 0..8 {
                let curr_x = x_pos + bit_idx;

                if curr_x >= (WIDTH as u8) {
                    // this should achieve a clipping behaviour
                    continue;
                }

                // get most significant bit
                let bit: bool = ((byte >> (7 - bit_idx)) & 1) == 1;

                if bit {
                    let curr_val = self.pixel_vals[curr_y as usize][curr_x as usize];
                    self.pixel_vals[curr_y as usize][curr_x as usize] = !curr_val;

                    self.screen_ui.flip_pixel(curr_x, curr_y, !curr_val);

                    pixel_turned_off |= curr_val;
                }
            }
        }

        return pixel_turned_off;
    }

    pub fn clear(&mut self) {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                self.pixel_vals[y][x] = false;
            }
        }
        self.screen_ui.clear();
    }

    pub fn update(&mut self) {
        self.screen_ui.update();
    }
}
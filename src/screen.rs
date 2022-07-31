pub const HEIGHT: u32 = 32;
pub const WIDTH: u32 = 64;

pub struct Screen {
    /// access pixel values using `pixel_vals[y][x]` (`x` = horizontal; `y` = vertical)
    frame_buffer: [[bool; (WIDTH as usize)]; (HEIGHT as usize)],
}

impl Screen {
    pub fn new() -> Screen {
        return Screen {
            frame_buffer: [[false; (WIDTH as usize)]; (HEIGHT as usize)],
        };
    }

    pub fn get_frame_buffer(&self) -> &[[bool; (WIDTH as usize)]; (HEIGHT as usize)] {
        return &self.frame_buffer;
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
                    let curr_val = self.frame_buffer[curr_y as usize][curr_x as usize];
                    self.frame_buffer[curr_y as usize][curr_x as usize] = !curr_val;

                    pixel_turned_off |= curr_val;
                }
            }
        }

        return pixel_turned_off;
    }

    pub fn clear(&mut self) {
        for y in 0..(HEIGHT as usize) {
            for x in 0..(WIDTH as usize) {
                self.frame_buffer[y][x] = false;
            }
        }
    }
}
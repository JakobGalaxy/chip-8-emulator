use std::time::{Duration, Instant};
use crate::keypad::Keypad;
use crate::screen;
use crate::stack::Stack;
use crate::screen::Screen;

/// specifies the ID of the VF register which is often used for flags
const FLAG_REG_ID: u8 = 0xF;

/// specifies the address where the font data is stored in memory
const FONT_START_ADDRESS: u16 = 0x050;

/// specifies the address where the program is stored in memory
pub const PROGRAM_START_ADDRESS: u16 = 0x200;

const INSTRUCTION_EXEC_DURATION: Duration = Duration::from_nanos(1_428_571); // 1_428_571

#[derive(Debug)]
pub enum Chip8Error {
    InstructionNotImplemented(String),

}

pub struct Chip8 {
    registers: [u8; 16],

    // position in memory
    program_counter: u16,

    // 4096 bytes of memory
    memory: [u8; 0x1000],

    /// specifies if the Y register is loaded into X before doing bit-shift operations or not
    assign_before_shift: bool,

    /// specifies whether it sets VF to 1 if I overflows from 0FFF to above 0x1000 (outside the normal addressing space) or not
    set_flag_on_index_overflow: bool,

    /// specifies if I is incremented during the FX55 (reg_dump) and FX65 (reg_load) instructions
    modify_index_on_dump_or_load: bool,

    stack: Stack,

    screen: Screen,

    keypad: Keypad,

    /// aka. the I register (used to point at locations in memory)
    index_reg: u16,

    delay_timer: u8,

    sound_timer: u8,

    playing_sound: bool,

    exec_time: Duration,

    last_exec: Instant,

    reached_end_of_file: bool,
}

impl Chip8 {
    pub fn new(assign_before_shift: bool, set_flag_on_index_overflow: bool, modify_index_on_dump_or_load: bool) -> Self {
        return Chip8 {
            registers: [0; 16],
            program_counter: PROGRAM_START_ADDRESS,
            memory: [0; 0x1000],
            assign_before_shift,
            set_flag_on_index_overflow,
            modify_index_on_dump_or_load,
            stack: Stack::new(),
            screen: Screen::new(),
            keypad: Keypad::new(),
            index_reg: 0x0,
            sound_timer: 0,
            delay_timer: 0,
            playing_sound: false,
            exec_time: Duration::new(0, 0),
            last_exec: Instant::now(),
            reached_end_of_file: false,
        };
    }

    /// **NOTE:** in comparison to the `add_const_to_x()` method, this one **does** set a carry flag, thus affecting the VF register
    fn add_y_to_x(&mut self, x_reg_id: u8, y_reg_id: u8) {
        let arg_1 = self.registers[x_reg_id as usize];
        let arg_2 = self.registers[y_reg_id as usize];

        let (value, carry) = arg_1.overflowing_add(arg_2);
        self.registers[x_reg_id as usize] = value;

        // set carry flag
        self.registers[FLAG_REG_ID as usize] = if carry { 1 } else { 0 };
    }

    /// **NOTE:** in comparison to the `add_y_to_x()` method, this one **does not** set a carry flag, thus not affecting the VF register
    fn add_const_to_x(&mut self, x_reg_id: u8, const_val: u8) {
        let arg_1 = self.registers[x_reg_id as usize];

        let (value, _) = arg_1.overflowing_add(const_val);
        self.registers[x_reg_id as usize] = value;
    }

    /// **NOTE:** if the operation results in an underflow (when there is a borrow), the VF register is set to 0, otherwise it is set to 1
    fn subtract_y_from_x(&mut self, x_reg_id: u8, y_reg_id: u8) {
        let arg_1 = self.registers[x_reg_id as usize];
        let arg_2 = self.registers[y_reg_id as usize];

        let (value, underflow) = arg_1.overflowing_sub(arg_2);

        self.registers[x_reg_id as usize] = value;

        // set underflow flag
        self.registers[FLAG_REG_ID as usize] = if underflow { 0 } else { 1 };
    }

    /// - **NOTE_1:** even though the method subtracts **`x`** from **`y`**, the result is still stored in **`x`**
    /// - **NOTE_2:** if the operation results in an underflow (when there is a borrow), the VF register is set to 0, otherwise it is set to 1
    fn subtract_x_from_y(&mut self, x_reg_id: u8, y_reg_id: u8) {
        let arg_1 = self.registers[x_reg_id as usize];
        let arg_2 = self.registers[y_reg_id as usize];

        let (value, underflow) = arg_2.overflowing_sub(arg_1);

        self.registers[x_reg_id as usize] = value;

        // set underflow flag
        self.registers[FLAG_REG_ID as usize] = if underflow { 0 } else { 1 };
    }

    fn assign_const_to_x(&mut self, x_reg_id: u8, const_val: u8) {
        self.registers[x_reg_id as usize] = const_val;
    }

    fn assign_y_to_x(&mut self, x_reg_id: u8, y_reg_id: u8) {
        self.registers[x_reg_id as usize] = self.registers[y_reg_id as usize];
    }

    fn bitwise_or_x_y(&mut self, x_reg_id: u8, y_reg_id: u8) {
        self.registers[x_reg_id as usize] |= self.registers[y_reg_id as usize];
    }

    fn bitwise_and_x_y(&mut self, x_reg_id: u8, y_reg_id: u8) {
        self.registers[x_reg_id as usize] &= self.registers[y_reg_id as usize];
    }

    fn bitwise_xor_x_y(&mut self, x_reg_id: u8, y_reg_id: u8) {
        self.registers[x_reg_id as usize] ^= self.registers[y_reg_id as usize];
    }

    /// shifts the X register 1 position to the right
    ///  - VF is set to the value of the least-significant-bit before the shift operation
    ///  - the `assign_before_shift` bool, which can be configured on creation, specifies whether the Y register is loaded into the X register before doing the shift operation
    fn right_bit_shift(&mut self, x_reg_id: u8, y_reg_id: u8) {
        if self.assign_before_shift {
            self.assign_y_to_x(x_reg_id, y_reg_id);
        }

        // set VF to LSB
        self.registers[FLAG_REG_ID as usize] = self.registers[x_reg_id as usize] & (0x01 as u8);

        self.registers[x_reg_id as usize] >>= 1;
    }

    /// shifts the X register 1 position to the left
    ///  - VF is set to the value of the most-significant-bit before the shift operation
    ///  - the `assign_before_shift` bool, which can be configured on creation, specifies whether the Y register is loaded into the X register before doing the shift operation
    fn left_bit_shift(&mut self, x_reg_id: u8, y_reg_id: u8) {
        if self.assign_before_shift {
            self.assign_y_to_x(x_reg_id, y_reg_id);
        }

        // set VF to LSB
        self.registers[FLAG_REG_ID as usize] = (self.registers[x_reg_id as usize] & (0x80 as u8)) >> 7;

        self.registers[x_reg_id as usize] <<= 1;
    }

    fn skip_if_x_equals_const(&mut self, x_reg_id: u8, const_val: u8) {
        let x_reg = &self.registers[x_reg_id as usize];
        if *x_reg == const_val {
            self.program_counter += 2;
        }
    }

    fn skip_if_x_not_equals_const(&mut self, x_reg_id: u8, const_val: u8) {
        let x_reg = &self.registers[x_reg_id as usize];
        if *x_reg != const_val {
            self.program_counter += 2;
        }
    }

    fn skip_if_x_equals_y(&mut self, x_reg_id: u8, y_reg_id: u8) {
        let x_reg = &self.registers[x_reg_id as usize];
        let y_reg = &self.registers[y_reg_id as usize];
        if *x_reg == *y_reg {
            self.program_counter += 2;
        }
    }
    fn skip_if_x_not_equals_y(&mut self, x_reg_id: u8, y_reg_id: u8) {
        let x_reg = &self.registers[x_reg_id as usize];
        let y_reg = &self.registers[y_reg_id as usize];
        if *x_reg != *y_reg {
            self.program_counter += 2;
        }
    }

    fn call_subroutine(&mut self, address: u16) {
        self.stack.push(self.program_counter);
        self.program_counter = address;
    }

    fn return_from_subroutine(&mut self) {
        let address: u16 = self.stack.pop();
        self.program_counter = address;
    }

    fn jump_to_address(&mut self, address: u16) {
        self.program_counter = address;
    }

    /// jumps to V0 + address
    fn jump_to_address_with_displacement(&mut self, address: u16) {
        self.jump_to_address(address + (self.registers[0x0] as u16));
    }

    fn set_index_reg(&mut self, address: u16) {
        self.index_reg = address;
    }

    /// **NOTE:** if the `set_flag_on_index_overflow` bool is set to `true`,
    /// then in case of the index register moving outside the normal addressing range (`0x1000`), VF is set to `1`
    fn add_x_to_index(&mut self, x_reg_id: u8) {
        self.index_reg += self.registers[x_reg_id as usize] as u16;

        // set overflow flag
        if self.set_flag_on_index_overflow && self.index_reg > 0x1000 {
            self.registers[FLAG_REG_ID as usize] = 1;
        }
    }

    fn set_index_to_char_font(&mut self, x_reg_id: u8) {
        // reduce to the least significant nibble
        let character = self.registers[x_reg_id as usize] & 0xF;
        self.index_reg = FONT_START_ADDRESS + (character as u16) * 5;
    }

    fn dump_registers_to_memory(&mut self, x_reg_id: u8) {
        let mut address: u16 = self.index_reg;
        for idx in 0..(x_reg_id + 1) {
            self.memory[address as usize] = self.registers[idx as usize];
            address += 1;
        }

        if self.modify_index_on_dump_or_load {
            self.index_reg = address;
        }
    }

    fn load_registers_from_memory(&mut self, x_reg_id: u8) {
        let mut address: u16 = self.index_reg;
        for idx in 0..(x_reg_id + 1) {
            self.registers[idx as usize] = self.memory[address as usize];
            address += 1;
        }

        if self.modify_index_on_dump_or_load {
            self.index_reg = address;
        }
    }

    fn display_sprite(&mut self, x_reg_id: u8, y_reg_id: u8, pixel_height: u8) {
        let x_pos = self.registers[x_reg_id as usize];
        let y_pos = self.registers[y_reg_id as usize];

        let sprite_data = &self.memory[(self.index_reg as usize)..(self.index_reg as usize) + (pixel_height as usize)];

        if self.screen.display_sprite(x_pos, y_pos, sprite_data) {
            self.registers[FLAG_REG_ID as usize] = 1;
        }
    }

    fn clear_screen(&mut self) {
        self.screen.clear();
    }

    fn set_x_to_delay_timer(&mut self, x_red_id: u8) {
        self.registers[x_red_id as usize] = self.delay_timer;
    }

    fn set_delay_timer_to_x(&mut self, x_reg_id: u8) {
        self.delay_timer = self.registers[x_reg_id as usize];
    }

    fn set_sound_timer_to_x(&mut self, x_reg_id: u8) {
        self.sound_timer = self.registers[x_reg_id as usize];
    }

    fn skip_if_key_pressed(&mut self, x_reg_id: u8) {
        let key_id: u8 = self.registers[x_reg_id as usize];
        if self.keypad.check_key_state(key_id) {
            self.program_counter += 2;
        }
    }

    fn skip_if_key_not_pressed(&mut self, x_reg_id: u8) {
        let key_id: u8 = self.registers[x_reg_id as usize];
        if !self.keypad.check_key_state(key_id) {
            self.program_counter += 2;
        }
    }

    fn await_keypress(&mut self, x_reg_id: u8) {
        let keypress: Option<u8> = self.keypad.get_keypress();

        if let Some(key_id) = keypress {
            self.registers[x_reg_id as usize] = key_id;
        } else {
            // repeat instruction until keypress is found
            self.program_counter -= 2;
        }
    }

    fn fetch_instruction(&mut self) -> u16 {
        // all opcodes are 2 bytes long and stored in big-endian format
        /*
        big-endian:
            - most significant byte of a word -> smallest memory address
            - least significant byte -> largest memory address
         */

        let byte_1 = self.memory[self.program_counter as usize] as u8; // most significant byte
        let byte_2 = self.memory[(self.program_counter + 1) as usize] as u8; // least significant byte

        return ((byte_1 as u16) << 8) | (byte_2 as u16);
    }

    /// returns `false` if there was nothing to execute (empty instruction)
    pub fn exec_next_instruction(&mut self) -> Result<(), Chip8Error> {
        let opcode = self.fetch_instruction();
        self.program_counter += 2;

        println!("time elapsed since last exec: {:?}; instruction: {:04x}", self.last_exec.elapsed(), opcode);
        self.last_exec = Instant::now();

        // opcode group (4 bit) -> first nibble
        let opcode_group: u8 = ((opcode & 0xF000) >> 12) as u8;

        // X register identifier (4 bit)
        let x_reg_id: u8 = ((opcode & 0x0F00) >> 8) as u8;

        // Y register identifier (4 bit)
        let y_reg_id: u8 = ((opcode & 0x00F0) >> 4) as u8;

        // opcode subgroup (4 bit)
        let opcode_subgroup: u8 = (opcode & 0x000F) as u8;

        // address (12 bit)
        let address: u16 = (opcode & 0x0FFF) as u16;

        // constant (8 bit)
        let const_val: u8 = (opcode & 0x00FF) as u8;

        // nibble constant (4 bit)
        let nibble_const_val: u8 = (opcode & 0x000F) as u8;

        match (opcode_group, x_reg_id, y_reg_id, opcode_subgroup) {
            // stop execution on empty instruction
            (0x0, 0x0, 0x0, 0x0) => self.reached_end_of_file = true,

            // basic math
            (0x8, _, _, 0x4) => self.add_y_to_x(x_reg_id, y_reg_id),
            (0x8, _, _, 0x5) => self.subtract_y_from_x(x_reg_id, y_reg_id),
            (0x8, _, _, 0x7) => self.subtract_x_from_y(x_reg_id, y_reg_id),
            (0x7, _, _, _) => self.add_const_to_x(x_reg_id, const_val),
            (0x6, _, _, _) => self.assign_const_to_x(x_reg_id, const_val),
            (0x8, _, _, 0x0) => self.assign_y_to_x(x_reg_id, y_reg_id),

            // bit-operations
            (0x8, _, _, 0x1) => self.bitwise_or_x_y(x_reg_id, y_reg_id),
            (0x8, _, _, 0x2) => self.bitwise_and_x_y(x_reg_id, y_reg_id),
            (0x8, _, _, 0x3) => self.bitwise_xor_x_y(x_reg_id, y_reg_id),
            (0x8, _, _, 0x6) => self.right_bit_shift(x_reg_id, y_reg_id),
            (0x8, _, _, 0xE) => self.left_bit_shift(x_reg_id, y_reg_id),

            // conditional-skips
            (0x3, _, _, _) => self.skip_if_x_equals_const(x_reg_id, const_val),
            (0x4, _, _, _) => self.skip_if_x_not_equals_const(x_reg_id, const_val),
            (0x5, _, _, 0x0) => self.skip_if_x_equals_y(x_reg_id, y_reg_id),
            (0x9, _, _, 0x0) => self.skip_if_x_not_equals_y(x_reg_id, y_reg_id),

            // flow-control
            (0x0, 0x0, 0xE, 0xE) => self.return_from_subroutine(),
            (0x1, _, _, _) => self.jump_to_address(address),
            (0x2, _, _, _) => self.call_subroutine(address),
            (0xB, _, _, _) => self.jump_to_address_with_displacement(address),

            // memory control
            (0xA, _, _, _) => self.set_index_reg(address),
            (0xF, _, 0x1, 0xE) => self.add_x_to_index(x_reg_id),
            (0xF, _, 0x2, 0x9) => self.set_index_to_char_font(x_reg_id),
            (0xF, _, 0x5, 0x5) => self.dump_registers_to_memory(x_reg_id),
            (0xF, _, 0x6, 0x5) => self.load_registers_from_memory(x_reg_id),

            // display
            (0xD, _, _, _) => self.display_sprite(x_reg_id, y_reg_id, nibble_const_val),
            (0x0, 0x0, 0xE, 0x0) => self.clear_screen(),

            // timers
            (0xF, _, 0x0, 0x7) => self.set_x_to_delay_timer(x_reg_id),
            (0xF, _, 0x1, 0x5) => self.set_delay_timer_to_x(x_reg_id),
            (0xF, _, 0x1, 0x8) => self.set_sound_timer_to_x(x_reg_id),

            // key input
            (0xE, _, 0x9, 0xE) => self.skip_if_key_pressed(x_reg_id),
            (0xE, _, 0xA, 0x1) => self.skip_if_key_not_pressed(x_reg_id),
            (0xF, _, 0x0, 0xA) => self.await_keypress(x_reg_id),

            _ => return Err(Chip8Error::InstructionNotImplemented(String::from(format!("there is no implementation for the instruction 0x{:04x} that was found at mem address 0x{:04x}!", opcode, self.program_counter - 2))))
        }

        return Ok(());
    }

    pub fn run_frame(&mut self, frame_duration: Duration) -> Result<(), Chip8Error> {
        // update timers
        self.decrement_timers();

        self.exec_time += frame_duration;

        // run instructions
        while self.exec_time >= INSTRUCTION_EXEC_DURATION {
            self.exec_next_instruction()?;
            self.exec_time -= INSTRUCTION_EXEC_DURATION;
        }

        return Ok(());
    }

    pub fn load_keypad(&mut self, keypad: Keypad) {
        self.keypad = keypad;
    }

    /// **NOTE:** should be executed 60 times a second -> every frame
    fn decrement_timers(&mut self) {
        // decrement delay timer
        self.delay_timer -= if self.delay_timer >= 1 { 1 } else { 0 };

        // decrement sound timer
        if self.sound_timer <= 1 {
            self.playing_sound = false;
            self.sound_timer = 0;
        } else {
            self.playing_sound = true;
            self.sound_timer -= 1;
        }
    }

    pub fn load_bytes_into_memory(&mut self, bytes: &Vec<u8>, address: u16) {
        for (idx, byte) in bytes.iter().enumerate() {
            self.memory[(address as usize) + idx] = *byte;
        }
    }

    pub fn load_opcode_into_memory(&mut self, opcode: u16, address: u16) {
        let byte_1 = ((opcode & 0xFF00) >> 8) as u8;
        let byte_2 = (opcode & 0x00FF) as u8;

        self.memory[address as usize] = byte_1;
        self.memory[(address + 1) as usize] = byte_2;
    }

    pub fn load_opcodes_into_memory(&mut self, opcodes: &Vec<u16>, mut address: u16) {
        for opcode in opcodes {
            self.load_opcode_into_memory(*opcode, address);
            address += 2;
        }
    }

    pub fn load_font_into_memory(&mut self, font_data: [[u8; 5]; 16]) {
        let mut address: u16 = FONT_START_ADDRESS;
        for character in font_data {
            for byte in character {
                self.memory[address as usize] = byte;
                address += 1;
            }
        }

        assert_eq!(address, 0xA0);
    }

    pub fn load_register(&mut self, reg_id: u8, value: u8) {
        self.registers[reg_id as usize] = value;
    }

    pub fn load_index_reg(&mut self, address: u16) {
        self.set_index_reg(address);
    }

    pub fn load_registers(&mut self, values: &[u8; 16]) {
        for (reg_id, value) in values.iter().enumerate() {
            self.registers[reg_id] = *value;
        }
    }

    pub fn playing_sound(&self) -> bool {
        return self.playing_sound;
    }

    pub fn reached_end_of_file(&self) -> bool {
        return self.reached_end_of_file;
    }

    pub fn reset_state(&mut self) {
        self.reached_end_of_file = false;
        self.program_counter = PROGRAM_START_ADDRESS;
        for val in self.registers.iter_mut() {
            *val = 0;
        }
    }

    pub fn get_frame_buffer(&self) -> &[[bool; (screen::WIDTH as usize)]; (screen::HEIGHT as usize)] {
        return self.screen.get_frame_buffer();
    }

    pub fn print_debug_info(&self) {
        println!("==== CHIP-8 CPU DEBUG INFO (START) ====");

        // output registers
        println!("REGISTERS:");
        for (i, reg) in self.registers.iter().enumerate() {
            println!("\t{:02}: 0x{:04x} = {:3}", i, reg, reg);
        }

        println!("==== CHIP-8 CPU DEBUG INFO (END) ====");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_emulator() -> Chip8 {
        let chip8 = Chip8::new(true, true, false);

        return chip8;
    }

    fn run_emulator(chip8: &mut Chip8) {
        let mut continue_execution: bool = true;
        while continue_execution {
            continue_execution = chip8.exec_next_instruction().expect("an error occurred during emulator execution");
        }
    }

    #[test]
    fn add_xy() {
        let mut chip8 = init_emulator();

        let val_1 = 5;
        let val_2 = 7;

        // load registers
        chip8.load_register(0, val_1);
        chip8.load_register(1, val_2);

        // load opcodes
        chip8.load_opcode_into_memory(0x8014, PROGRAM_START_ADDRESS);
        run_emulator(&mut chip8);

        // verify result
        assert_eq!(chip8.registers[0], val_1 + val_2, "failed to correctly add the two registers; a: {}, b: {}, result: {}", val_1, val_2, chip8.registers[0]);

        let vf_register = &chip8.registers[FLAG_REG_ID as usize];
        assert_eq!(*vf_register, 0, "failed to correctly set the carry bit; VF register: 0x{:02x}", vf_register);
    }

    #[test]
    fn add_xy_with_carry() {
        let mut chip8 = init_emulator();

        let val_1 = 1;
        let val_2 = 255;

        // load registers
        chip8.load_register(0, val_1);
        chip8.load_register(1, val_2);

        // load opcodes
        chip8.load_opcode_into_memory(0x8014, PROGRAM_START_ADDRESS);
        run_emulator(&mut chip8);

        // verify result
        assert_eq!(chip8.registers[0], 0, "failed to correctly add the two registers; a: {}, b: {}, result: {}", val_1, val_2, chip8.registers[0]);

        let vf_register = &chip8.registers[FLAG_REG_ID as usize];
        assert_eq!(*vf_register, 1, "failed to correctly set the carry bit; VF register: 0x{:02x}", vf_register);
    }

    #[test]
    fn add_const_to_x() {
        let mut chip8 = init_emulator();

        let val_1 = 5;
        let val_2 = 7;

        // load registers
        chip8.load_register(0, val_1);

        // load opcodes
        let opcode: u16 = (0x7000 as u16) | (val_2 as u16);
        chip8.load_opcode_into_memory(opcode, PROGRAM_START_ADDRESS);
        run_emulator(&mut chip8);

        // verify result
        assert_eq!(chip8.registers[0], val_1 + val_2, "failed to correctly add a constant and a register; a: {}, b: {}, result: {}", val_1, val_2, chip8.registers[0]);
    }

    #[test]
    fn subtract_y_from_x() {
        let mut chip8 = init_emulator();

        let val_1 = 8;
        let val_2 = 3;

        // load registers
        chip8.load_register(0, val_1);
        chip8.load_register(1, val_2);

        // load opcodes
        chip8.load_opcode_into_memory(0x8015, PROGRAM_START_ADDRESS);
        run_emulator(&mut chip8);

        // verify result
        assert_eq!(chip8.registers[0], val_1 - val_2, "failed to correctly subtract the two registers (result = a - b); a: {}, b: {}, result: {}", val_1, val_2, chip8.registers[0]);

        let vf_register = &chip8.registers[FLAG_REG_ID as usize];
        assert_eq!(*vf_register, 1, "failed to correctly set the underflow bit; VF register: 0x{:02x}", vf_register);
    }

    #[test]
    fn subtract_y_from_x_with_underflow() {
        let mut chip8 = init_emulator();

        let val_1 = 8;
        let val_2 = 10;

        // load registers
        chip8.load_register(0, val_1);
        chip8.load_register(1, val_2);

        // load opcodes
        chip8.load_opcode_into_memory(0x8015, PROGRAM_START_ADDRESS);
        run_emulator(&mut chip8);

        // verify result
        assert_eq!(chip8.registers[0], 254, "failed to correctly subtract the two registers (result = a - b); a: {}, b: {}, result: {}", val_1, val_2, chip8.registers[0]);

        let vf_register = &chip8.registers[FLAG_REG_ID as usize];
        assert_eq!(*vf_register, 0, "failed to correctly set the underflow bit; VF register: 0x{:02x}", vf_register);
    }

    #[test]
    fn subtract_x_from_y() {
        let mut chip8 = init_emulator();

        let val_1 = 3;
        let val_2 = 8;

        // load registers
        chip8.load_register(0, val_1);
        chip8.load_register(1, val_2);

        // load opcodes
        chip8.load_opcode_into_memory(0x8017, PROGRAM_START_ADDRESS);
        run_emulator(&mut chip8);

        // verify result
        assert_eq!(chip8.registers[0], val_2 - val_1, "failed to correctly subtract the two registers (result = b - a); a: {}, b: {}, result: {}", val_1, val_2, chip8.registers[0]);

        let vf_register = &chip8.registers[FLAG_REG_ID as usize];
        assert_eq!(*vf_register, 1, "failed to correctly set the underflow bit; VF register: 0x{:02x}", vf_register);
    }

    #[test]
    fn subtract_x_from_y_with_underflow() {
        let mut chip8 = init_emulator();

        let val_1 = 10;
        let val_2 = 8;

        // load registers
        chip8.load_register(0, val_1);
        chip8.load_register(1, val_2);

        // load opcodes
        chip8.load_opcode_into_memory(0x8017, PROGRAM_START_ADDRESS);
        run_emulator(&mut chip8);

        // verify result
        assert_eq!(chip8.registers[0], 254, "failed to correctly subtract the two registers (result = b - a); a: {}, b: {}, result: {}", val_1, val_2, chip8.registers[0]);

        let vf_register = &chip8.registers[FLAG_REG_ID as usize];
        assert_eq!(*vf_register, 0, "failed to correctly set the underflow bit; VF register: 0x{:02x}", vf_register);
    }

    #[test]
    fn assign_const_to_x() {
        let mut chip8 = init_emulator();

        let val_1: u8 = 0x15;

        // load opcodes
        let opcode: u16 = (0x6000 as u16) | (val_1 as u16);
        chip8.load_opcode_into_memory(opcode, PROGRAM_START_ADDRESS);
        run_emulator(&mut chip8);

        // verify result
        assert_eq!(chip8.registers[0], val_1, "failed to correctly assign constant to register; constant: {}, reg: {}", val_1, chip8.registers[0]);
    }

    #[test]
    fn assign_y_to_x() {
        let mut chip8 = init_emulator();

        let val_1 = 10;

        // load registers
        chip8.load_register(1, val_1);

        // load opcodes
        chip8.load_opcode_into_memory(0x8010, PROGRAM_START_ADDRESS);
        run_emulator(&mut chip8);

        // verify result
        assert_eq!(chip8.registers[0], val_1, "failed to correctly assign register y to register x; reg_y: {}, reg_x: {}", val_1, chip8.registers[0]);
    }

    #[test]
    fn bitwise_or_x_y() {
        let mut chip8 = init_emulator();

        let val_1 = 10;
        let val_2 = 15;

        // load registers
        chip8.load_register(0, val_1);
        chip8.load_register(1, val_2);

        // load opcodes
        chip8.load_opcode_into_memory(0x8011, PROGRAM_START_ADDRESS);
        run_emulator(&mut chip8);

        // verify result
        assert_eq!(chip8.registers[0], (val_1 | val_2), "failed to correctly perform the bitwise OR operation on 2 registers; val_1: {}, val_2: {}, result: {}", val_1, val_2, chip8.registers[0]);
    }

    #[test]
    fn bitwise_and_x_y() {
        let mut chip8 = init_emulator();

        let val_1 = 64;
        let val_2 = 15;

        // load registers
        chip8.load_register(0, val_1);
        chip8.load_register(1, val_2);

        // load opcodes
        chip8.load_opcode_into_memory(0x8012, PROGRAM_START_ADDRESS);
        run_emulator(&mut chip8);

        // verify result
        assert_eq!(chip8.registers[0], (val_1 & val_2), "failed to correctly perform the bitwise AND operation on 2 registers; val_1: {}, val_2: {}, result: {}", val_1, val_2, chip8.registers[0]);
    }

    #[test]
    fn bitwise_xor_x_y() {
        let mut chip8 = init_emulator();

        let val_1 = 65;
        let val_2 = 15;

        // load registers
        chip8.load_register(0, val_1);
        chip8.load_register(1, val_2);

        // load opcodes
        chip8.load_opcode_into_memory(0x8013, PROGRAM_START_ADDRESS);
        run_emulator(&mut chip8);

        // verify result
        assert_eq!(chip8.registers[0], (val_1 ^ val_2), "failed to correctly perform the bitwise XOR operation on 2 registers; val_1: {}, val_2: {}, result: {}", val_1, val_2, chip8.registers[0]);
    }

    #[test]
    fn right_bit_shift() {
        let mut chip8 = init_emulator();

        let val_1 = 65;

        // load registers
        chip8.load_register(1, val_1);

        // load opcodes
        chip8.load_opcode_into_memory(0x8016, PROGRAM_START_ADDRESS);
        run_emulator(&mut chip8);

        // verify result
        assert_eq!(chip8.registers[0], val_1 >> 1, "failed to correctly perform the right bit-shift operation; val_1: {}, result: {}", val_1, chip8.registers[0]);

        let vf_register = &chip8.registers[FLAG_REG_ID as usize];
        assert_eq!(*vf_register, 1, "failed to correctly load the LSB into VF; VF register: 0x{:02x}", vf_register);
    }

    #[test]
    fn left_bit_shift() {
        let mut chip8 = init_emulator();

        let val_1 = 255;

        // load registers
        chip8.load_register(1, val_1);

        // load opcodes
        chip8.load_opcode_into_memory(0x801E, PROGRAM_START_ADDRESS);
        run_emulator(&mut chip8);

        // verify result
        assert_eq!(chip8.registers[0], val_1 << 1, "failed to correctly perform the left bit-shift operation; val_1: {}, result: {}", val_1, chip8.registers[0]);

        let vf_register = &chip8.registers[FLAG_REG_ID as usize];
        assert_eq!(*vf_register, 1, "failed to correctly load the LSB into VF; VF register: 0x{:02x}", vf_register);
    }

    #[test]
    fn skip_if_x_equals_const() {
        let mut chip8 = init_emulator();

        let val_1 = 5;

        // load registers
        chip8.load_register(0, val_1);

        // load opcodes
        let opcode: u16 = (0x3000 as u16) | (val_1 as u16);
        chip8.load_opcode_into_memory(opcode, PROGRAM_START_ADDRESS);
        // if the skip fails, V0 is set to 0x11
        chip8.load_opcode_into_memory(0x6011, PROGRAM_START_ADDRESS + 2);
        run_emulator(&mut chip8);

        // verify result
        assert_eq!(chip8.registers[0], val_1, "failed to correctly perform the if(VX == NN) operation");
    }

    #[test]
    fn skip_if_x_not_equals_const() {
        let mut chip8 = init_emulator();

        let val_1 = 5;

        // load registers
        chip8.load_register(0, val_1);

        // load opcodes
        let opcode: u16 = (0x4000 as u16) | ((val_1 + 1) as u16);
        chip8.load_opcode_into_memory(opcode, PROGRAM_START_ADDRESS);
        // if the skip fails, V0 is set to 0x11
        chip8.load_opcode_into_memory(0x6011, PROGRAM_START_ADDRESS + 2);
        run_emulator(&mut chip8);

        // verify result
        assert_eq!(chip8.registers[0], val_1, "failed to correctly perform the if(VX != NN) operation");
    }

    #[test]
    fn skip_if_x_equals_y() {
        let mut chip8 = init_emulator();

        let val_1 = 5;

        // load registers
        chip8.load_register(0, val_1);
        chip8.load_register(1, val_1);

        // load opcodes
        chip8.load_opcode_into_memory(0x5010, PROGRAM_START_ADDRESS);
        // if the skip fails, V0 is set to 0x11
        chip8.load_opcode_into_memory(0x6011, PROGRAM_START_ADDRESS + 2);
        run_emulator(&mut chip8);

        // verify result
        assert_eq!(chip8.registers[0], val_1, "failed to correctly perform the if(VX == VY) operation");
    }

    #[test]
    fn skip_if_x_not_equals_y() {
        let mut chip8 = init_emulator();

        let val_1 = 5;

        // load registers
        chip8.load_register(0, val_1);
        chip8.load_register(1, val_1 + 1);

        // load opcodes
        chip8.load_opcode_into_memory(0x9010, PROGRAM_START_ADDRESS);
        // if the skip fails, V0 is set to 0x11
        chip8.load_opcode_into_memory(0x6011, PROGRAM_START_ADDRESS + 2);
        run_emulator(&mut chip8);

        // verify result
        assert_eq!(chip8.registers[0], val_1, "failed to correctly perform the if(VX != VY) operation");
    }

    #[test]
    fn call_and_return_from_subroutine() {
        let mut chip8 = init_emulator();

        let val_1 = 5;
        let val_2 = 7;

        // load registers
        chip8.load_register(0, val_1);
        chip8.load_register(1, val_2);

        // load opcodes
        let main_opcodes: Vec<u16> = vec!(0x2300, 0x8014);
        chip8.load_opcodes_into_memory(&main_opcodes, PROGRAM_START_ADDRESS);

        let subroutine_opcodes: Vec<u16> = vec!(0x8104, 0x00EE);
        chip8.load_opcodes_into_memory(&subroutine_opcodes, 0x300);

        run_emulator(&mut chip8);

        // verify result
        assert_eq!(chip8.registers[1], val_1 + val_2, "failed to correctly call subroutine");

        assert_eq!(chip8.registers[0], val_1 * 2 + val_2, "failed to correctly return from subroutine");
    }

    #[test]
    fn jump_to_address() {
        let mut chip8 = init_emulator();

        let val_1 = 5;
        let val_2 = 7;

        // load registers
        chip8.load_register(0, val_1);
        chip8.load_register(1, val_2);

        // load opcodes
        chip8.load_opcode_into_memory(0x2300, PROGRAM_START_ADDRESS);
        chip8.load_opcode_into_memory(0x8104, 0x300);

        run_emulator(&mut chip8);

        // verify result
        assert_eq!(chip8.registers[1], val_1 + val_2, "failed to correctly execute jump");
    }

    #[test]
    fn jump_to_address_with_displacement() {
        let mut chip8 = init_emulator();

        let val_1 = 5;
        let val_2 = 7;

        // load registers
        chip8.load_register(0, val_1);
        chip8.load_register(1, val_2);

        // load opcodes
        chip8.load_opcode_into_memory(0xB2FB, PROGRAM_START_ADDRESS);
        chip8.load_opcode_into_memory(0x8104, 0x300);

        run_emulator(&mut chip8);

        // verify result
        assert_eq!(chip8.registers[1], val_1 + val_2, "failed to correctly execute jump");
    }

    #[test]
    fn set_index_reg() {
        let mut chip8 = init_emulator();

        let val_1: u16 = 5;

        // load opcodes
        let opcode: u16 = (0xA000 as u16) | val_1;
        chip8.load_opcode_into_memory(opcode, PROGRAM_START_ADDRESS);

        run_emulator(&mut chip8);

        // verify result
        assert_eq!(chip8.index_reg, val_1, "failed to correctly set the index register; index_reg: {}", chip8.index_reg);
    }

    #[test]
    fn add_x_to_index() {
        let mut chip8 = init_emulator();

        let val_1: u16 = 5;
        let val_2: u8 = 7;

        // load registers
        chip8.load_index_reg(val_1);
        chip8.load_register(0, val_2);

        // load opcodes
        chip8.load_opcode_into_memory(0xF01E, PROGRAM_START_ADDRESS);

        run_emulator(&mut chip8);

        // verify result
        assert_eq!(chip8.index_reg, val_1 + (val_2 as u16), "failed to correctly add to X to index register; index_reg: {}", chip8.index_reg);
    }

    #[test]
    fn set_index_to_char_font() {
        let mut chip8 = init_emulator();

        let val_1: u8 = 0xF;

        // load registers
        chip8.load_register(0, val_1);

        // load opcodes
        chip8.load_opcode_into_memory(0xF029, PROGRAM_START_ADDRESS);

        run_emulator(&mut chip8);

        // verify result
        assert_eq!(chip8.index_reg, FONT_START_ADDRESS + (15 * 5), "failed to correctly set the index register to the font location; index_reg: 0x{:04x}; character: 0x{:02x}", chip8.index_reg, val_1);
    }

    #[test]
    fn dump_registers_to_memory() {
        let mut chip8 = init_emulator();

        let vals: [u8; 16] = [16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1];

        // load registers
        chip8.load_registers(&vals);
        chip8.index_reg = 0x300;

        // load opcodes
        chip8.load_opcode_into_memory(0xFF55, PROGRAM_START_ADDRESS);

        run_emulator(&mut chip8);

        // verify result
        for (idx, val) in vals.iter().enumerate() {
            assert_eq!(chip8.memory[(chip8.index_reg as usize) + idx], *val, "failed to correctly dump register V{:1X} into memory", idx);
        }
    }

    #[test]
    fn load_registers_from_memory() {
        let mut chip8 = init_emulator();

        let vals: Vec<u8> = vec!(16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1);

        // load registers
        chip8.index_reg = 0x300;

        // load memory
        chip8.load_bytes_into_memory(&vals, chip8.index_reg);

        // load opcodes
        chip8.load_opcode_into_memory(0xFF65, PROGRAM_START_ADDRESS);

        run_emulator(&mut chip8);

        // verify result
        for (idx, val) in vals.iter().enumerate() {
            assert_eq!(chip8.registers[idx], *val, "failed to correctly load register V{:1X} from memory", idx);
        }
    }
}
use crate::stack::Stack;

/// specifies the ID of the VF register which is often used for flags
const FLAG_REG_ID: u8 = 0xF;

pub struct CPU {
    registers: [u8; 16],

    // position in memory
    program_counter: u16,

    // 4096 bytes of memory
    memory: [u8; 0x1000],

    /// specifies if the Y register is loaded into X before doing bit-shift operations or not
    assign_before_shift: bool,

    stack: Stack,
}

impl CPU {
    pub fn new(assign_before_shift: bool) -> CPU {
        return CPU {
            registers: [0; 16],
            program_counter: 0x0,
            memory: [0; 0x1000],
            assign_before_shift,
            stack: Stack::new(),
        };
    }

    fn read_opcode(&self) -> u16 {
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

    pub fn load_register(&mut self, reg_id: u8, value: u8) {
        self.registers[reg_id as usize] = value;
    }

    pub fn load_registers(&mut self, values: &[u8; 16]) {
        for (reg_id, value) in values.iter().enumerate() {
            self.registers[reg_id] = *value;
        }
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

    pub fn run(&mut self) {
        loop {
            let opcode: u16 = self.read_opcode();
            self.program_counter += 2;

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

            match (opcode_group, x_reg_id, y_reg_id, opcode_subgroup) {
                (0, 0, 0, 0) => return (),

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
                // TODO: unit tests
                (0x0, 0x0, 0xE, 0xE) => self.return_from_subroutine(),
                (0x1, _, _, _) => self.jump_to_address(address),
                (0x2, _, _, _) => self.call_subroutine(address),
                (0xB, _, _, _) => self.jump_to_address_with_displacement(address),

                _ => todo!("opcode {:04x} is not implemented yet!", opcode)
            }
        }
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

    #[test]
    fn add_xy() {
        let mut cpu = CPU::new(true);

        let val_1 = 5;
        let val_2 = 7;

        // load registers
        cpu.load_register(0, val_1);
        cpu.load_register(1, val_2);

        // load opcodes
        cpu.load_opcode_into_memory(0x8014, 0x0);
        cpu.run();

        // verify result
        assert_eq!(cpu.registers[0], val_1 + val_2, "failed to correctly add the two registers; a: {}, b: {}, result: {}", val_1, val_2, cpu.registers[0]);

        let vf_register = &cpu.registers[FLAG_REG_ID as usize];
        assert_eq!(*vf_register, 0, "failed to correctly set the carry bit; VF register: 0x{:02x}", vf_register);
    }

    #[test]
    fn add_xy_with_carry() {
        let mut cpu = CPU::new(true);

        let val_1 = 1;
        let val_2 = 255;

        // load registers
        cpu.load_register(0, val_1);
        cpu.load_register(1, val_2);

        // load opcodes
        cpu.load_opcode_into_memory(0x8014, 0x0);
        cpu.run();

        // verify result
        assert_eq!(cpu.registers[0], 0, "failed to correctly add the two registers; a: {}, b: {}, result: {}", val_1, val_2, cpu.registers[0]);

        let vf_register = &cpu.registers[FLAG_REG_ID as usize];
        assert_eq!(*vf_register, 1, "failed to correctly set the carry bit; VF register: 0x{:02x}", vf_register);
    }

    #[test]
    fn add_const_to_x() {
        let mut cpu = CPU::new(true);

        let val_1 = 5;
        let val_2 = 7;

        // load registers
        cpu.load_register(0, val_1);

        // load opcodes
        let opcode: u16 = (0x7000 as u16) | (val_2 as u16);
        cpu.load_opcode_into_memory(opcode, 0x0);
        cpu.run();

        // verify result
        assert_eq!(cpu.registers[0], val_1 + val_2, "failed to correctly add a constant and a register; a: {}, b: {}, result: {}", val_1, val_2, cpu.registers[0]);
    }

    #[test]
    fn subtract_y_from_x() {
        let mut cpu = CPU::new(true);

        let val_1 = 8;
        let val_2 = 3;

        // load registers
        cpu.load_register(0, val_1);
        cpu.load_register(1, val_2);

        // load opcodes
        cpu.load_opcode_into_memory(0x8015, 0x0);
        cpu.run();

        // verify result
        assert_eq!(cpu.registers[0], val_1 - val_2, "failed to correctly subtract the two registers (result = a - b); a: {}, b: {}, result: {}", val_1, val_2, cpu.registers[0]);

        let vf_register = &cpu.registers[FLAG_REG_ID as usize];
        assert_eq!(*vf_register, 1, "failed to correctly set the underflow bit; VF register: 0x{:02x}", vf_register);
    }

    #[test]
    fn subtract_y_from_x_with_underflow() {
        let mut cpu = CPU::new(true);

        let val_1 = 8;
        let val_2 = 10;

        // load registers
        cpu.load_register(0, val_1);
        cpu.load_register(1, val_2);

        // load opcodes
        cpu.load_opcode_into_memory(0x8015, 0x0);
        cpu.run();

        // verify result
        assert_eq!(cpu.registers[0], 254, "failed to correctly subtract the two registers (result = a - b); a: {}, b: {}, result: {}", val_1, val_2, cpu.registers[0]);

        let vf_register = &cpu.registers[FLAG_REG_ID as usize];
        assert_eq!(*vf_register, 0, "failed to correctly set the underflow bit; VF register: 0x{:02x}", vf_register);
    }

    #[test]
    fn subtract_x_from_y() {
        let mut cpu = CPU::new(true);

        let val_1 = 3;
        let val_2 = 8;

        // load registers
        cpu.load_register(0, val_1);
        cpu.load_register(1, val_2);

        // load opcodes
        cpu.load_opcode_into_memory(0x8017, 0x0);
        cpu.run();

        // verify result
        assert_eq!(cpu.registers[0], val_2 - val_1, "failed to correctly subtract the two registers (result = b - a); a: {}, b: {}, result: {}", val_1, val_2, cpu.registers[0]);

        let vf_register = &cpu.registers[FLAG_REG_ID as usize];
        assert_eq!(*vf_register, 1, "failed to correctly set the underflow bit; VF register: 0x{:02x}", vf_register);
    }

    #[test]
    fn subtract_x_from_y_with_underflow() {
        let mut cpu = CPU::new(true);

        let val_1 = 10;
        let val_2 = 8;

        // load registers
        cpu.load_register(0, val_1);
        cpu.load_register(1, val_2);

        // load opcodes
        cpu.load_opcode_into_memory(0x8017, 0x0);
        cpu.run();

        // verify result
        assert_eq!(cpu.registers[0], 254, "failed to correctly subtract the two registers (result = b - a); a: {}, b: {}, result: {}", val_1, val_2, cpu.registers[0]);

        let vf_register = &cpu.registers[FLAG_REG_ID as usize];
        assert_eq!(*vf_register, 0, "failed to correctly set the underflow bit; VF register: 0x{:02x}", vf_register);
    }

    #[test]
    fn assign_const_to_x() {
        let mut cpu = CPU::new(true);

        let val_1: u8 = 0x15;

        // load opcodes
        let opcode: u16 = (0x6000 as u16) | (val_1 as u16);
        cpu.load_opcode_into_memory(opcode, 0x0);
        cpu.run();

        // verify result
        assert_eq!(cpu.registers[0], val_1, "failed to correctly assign constant to register; constant: {}, reg: {}", val_1, cpu.registers[0]);
    }

    #[test]
    fn assign_y_to_x() {
        let mut cpu = CPU::new(true);

        let val_1 = 10;

        // load registers
        cpu.load_register(1, val_1);

        // load opcodes
        cpu.load_opcode_into_memory(0x8010, 0x0);
        cpu.run();

        // verify result
        assert_eq!(cpu.registers[0], val_1, "failed to correctly assign register y to register x; reg_y: {}, reg_x: {}", val_1, cpu.registers[0]);
    }

    #[test]
    fn bitwise_or_x_y() {
        let mut cpu = CPU::new(true);

        let val_1 = 10;
        let val_2 = 15;

        // load registers
        cpu.load_register(0, val_1);
        cpu.load_register(1, val_2);

        // load opcodes
        cpu.load_opcode_into_memory(0x8011, 0x0);
        cpu.run();

        // verify result
        assert_eq!(cpu.registers[0], (val_1 | val_2), "failed to correctly perform the bitwise OR operation on 2 registers; val_1: {}, val_2: {}, result: {}", val_1, val_2, cpu.registers[0]);
    }

    #[test]
    fn bitwise_and_x_y() {
        let mut cpu = CPU::new(true);

        let val_1 = 64;
        let val_2 = 15;

        // load registers
        cpu.load_register(0, val_1);
        cpu.load_register(1, val_2);

        // load opcodes
        cpu.load_opcode_into_memory(0x8012, 0x0);
        cpu.run();

        // verify result
        assert_eq!(cpu.registers[0], (val_1 & val_2), "failed to correctly perform the bitwise AND operation on 2 registers; val_1: {}, val_2: {}, result: {}", val_1, val_2, cpu.registers[0]);
    }

    #[test]
    fn bitwise_xor_x_y() {
        let mut cpu = CPU::new(true);

        let val_1 = 65;
        let val_2 = 15;

        // load registers
        cpu.load_register(0, val_1);
        cpu.load_register(1, val_2);

        // load opcodes
        cpu.load_opcode_into_memory(0x8013, 0x0);
        cpu.run();

        // verify result
        assert_eq!(cpu.registers[0], (val_1 ^ val_2), "failed to correctly perform the bitwise XOR operation on 2 registers; val_1: {}, val_2: {}, result: {}", val_1, val_2, cpu.registers[0]);
    }

    #[test]
    fn right_bit_shift() {
        let mut cpu = CPU::new(true);

        let val_1 = 65;

        // load registers
        cpu.load_register(1, val_1);

        // load opcodes
        cpu.load_opcode_into_memory(0x8016, 0x0);
        cpu.run();

        // verify result
        assert_eq!(cpu.registers[0], val_1 >> 1, "failed to correctly perform the right bit-shift operation; val_1: {}, result: {}", val_1, cpu.registers[0]);

        let vf_register = &cpu.registers[FLAG_REG_ID as usize];
        assert_eq!(*vf_register, 1, "failed to correctly load the LSB into VF; VF register: 0x{:02x}", vf_register);
    }

    #[test]
    fn left_bit_shift() {
        let mut cpu = CPU::new(true);

        let val_1 = 255;

        // load registers
        cpu.load_register(1, val_1);

        // load opcodes
        cpu.load_opcode_into_memory(0x801E, 0x0);
        cpu.run();

        // verify result
        assert_eq!(cpu.registers[0], val_1 << 1, "failed to correctly perform the left bit-shift operation; val_1: {}, result: {}", val_1, cpu.registers[0]);

        let vf_register = &cpu.registers[FLAG_REG_ID as usize];
        assert_eq!(*vf_register, 1, "failed to correctly load the LSB into VF; VF register: 0x{:02x}", vf_register);
    }

    #[test]
    fn skip_if_x_equals_const() {
        let mut cpu = CPU::new(true);

        let val_1 = 5;

        // load registers
        cpu.load_register(0, val_1);

        // load opcodes
        let opcode: u16 = (0x3000 as u16) | (val_1 as u16);
        cpu.load_opcode_into_memory(opcode, 0x0);
        // if the skip fails, V0 is set to 0x11
        cpu.load_opcode_into_memory(0x6011, 0x2);
        cpu.run();

        // verify result
        assert_eq!(cpu.registers[0], val_1, "failed to correctly perform the if(VX == NN) operation");
    }

    #[test]
    fn skip_if_x_not_equals_const() {
        let mut cpu = CPU::new(true);

        let val_1 = 5;

        // load registers
        cpu.load_register(0, val_1);

        // load opcodes
        let opcode: u16 = (0x4000 as u16) | ((val_1 + 1) as u16);
        cpu.load_opcode_into_memory(opcode, 0x0);
        // if the skip fails, V0 is set to 0x11
        cpu.load_opcode_into_memory(0x6011, 0x2);
        cpu.run();

        // verify result
        assert_eq!(cpu.registers[0], val_1, "failed to correctly perform the if(VX != NN) operation");
    }

    #[test]
    fn skip_if_x_equals_y() {
        let mut cpu = CPU::new(true);

        let val_1 = 5;

        // load registers
        cpu.load_register(0, val_1);
        cpu.load_register(1, val_1);

        // load opcodes
        cpu.load_opcode_into_memory(0x5010, 0x0);
        // if the skip fails, V0 is set to 0x11
        cpu.load_opcode_into_memory(0x6011, 0x2);
        cpu.run();

        // verify result
        assert_eq!(cpu.registers[0], val_1, "failed to correctly perform the if(VX == VY) operation");
    }

    #[test]
    fn skip_if_x_not_equals_y() {
        let mut cpu = CPU::new(true);

        let val_1 = 5;

        // load registers
        cpu.load_register(0, val_1);
        cpu.load_register(1, val_1 + 1);

        // load opcodes
        cpu.load_opcode_into_memory(0x9010, 0x0);
        // if the skip fails, V0 is set to 0x11
        cpu.load_opcode_into_memory(0x6011, 0x2);
        cpu.run();

        // verify result
        assert_eq!(cpu.registers[0], val_1, "failed to correctly perform the if(VX != VY) operation");
    }

    #[test]
    fn call_and_return_from_subroutine() {
        let mut cpu = CPU::new(true);

        let val_1 = 5;
        let val_2 = 7;

        // load registers
        cpu.load_register(0, val_1);
        cpu.load_register(1, val_2);

        // load opcodes
        let main_opcodes: Vec<u16> = vec!(0x2100, 0x8014);
        cpu.load_opcodes_into_memory(&main_opcodes, 0x0);

        let subroutine_opcodes: Vec<u16> = vec!(0x8104, 0x00EE);
        cpu.load_opcodes_into_memory(&subroutine_opcodes, 0x100);

        cpu.run();

        // verify result
        assert_eq!(cpu.registers[1], val_1 + val_2, "failed to correctly call subroutine");

        assert_eq!(cpu.registers[0], val_1 * 2 + val_2, "failed to correctly return from subroutine");
    }

    #[test]
    fn jump_to_address() {
        let mut cpu = CPU::new(true);

        let val_1 = 5;
        let val_2 = 7;

        // load registers
        cpu.load_register(0, val_1);
        cpu.load_register(1, val_2);

        // load opcodes
        cpu.load_opcode_into_memory(0x2100, 0x0);
        cpu.load_opcode_into_memory(0x8104, 0x100);

        cpu.run();

        // verify result
        assert_eq!(cpu.registers[1], val_1 + val_2, "failed to correctly execute jump");
    }

    #[test]
    fn jump_to_address_with_displacement() {
        let mut cpu = CPU::new(true);

        let val_1 = 5;
        let val_2 = 7;

        // load registers
        cpu.load_register(0, val_1);
        cpu.load_register(1, val_2);

        // load opcodes
        cpu.load_opcode_into_memory(0xB0FB, 0x0);
        cpu.load_opcode_into_memory(0x8104, 0x100);

        cpu.run();

        // verify result
        assert_eq!(cpu.registers[1], val_1 + val_2, "failed to correctly execute jump");
    }
}
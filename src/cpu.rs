pub struct CPU {
    registers: [u8; 16],
    program_counter: u16,

    // position in memory
    memory: [u8; 0x1000],
}

impl CPU {
    pub fn new() -> CPU {
        return CPU {
            registers: [0; 16],
            program_counter: 0x0,
            memory: [0; 0x1000],
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
        self.registers[0xF] = if carry { 1 } else { 0 };
    }

    /// **NOTE:** in comparison to the `add_y_to_x()` method, this one **does not** set a carry flag, thus not affecting the VF register
    fn add_const_to_x(&mut self, x_reg_id: u8, const_val: u8) {
        let arg_1 = self.registers[x_reg_id as usize];

        let (value, _) = arg_1.overflowing_add(const_val);
        self.registers[x_reg_id as usize] = value;
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
                (0x8, _, _, 0x4) => self.add_y_to_x(x_reg_id, y_reg_id),
                (0x7, _, _, _) => self.add_const_to_x(x_reg_id, const_val),
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
        let mut cpu = CPU::new();

        let val_1 = 5;
        let val_2 = 7;

        // load registers
        cpu.load_register(0, val_1);
        cpu.load_register(1, val_2);

        // load opcodes
        cpu.load_opcode_into_memory(0x8014, 0x0);
        cpu.run();

        // verify result
        assert_eq!(cpu.registers[0], 12, "failed to correctly add the two registers; a: {}, b: {}, result: {}", val_1, val_2, cpu.registers[0]);

        let vf_register = &cpu.registers[0xF];
        assert_eq!(*vf_register, 0, "failed to correctly set the carry bit; VF register: 0x{:02x}", vf_register);
    }

    #[test]
    fn add_xy_with_carry() {
        let mut cpu = CPU::new();

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

        let vf_register = &cpu.registers[0xF];
        assert_eq!(*vf_register, 1, "failed to correctly set the carry bit; VF register: 0x{:02x}", vf_register);
    }

    #[test]
    fn add_const_to_x() {
        let mut cpu = CPU::new();

        let val_1 = 5;
        let val_2 = 7;

        // load registers
        cpu.load_register(0, val_1);

        // load opcodes
        let opcode: u16 = (0x7000 as u16) | val_2;
        cpu.load_opcode_into_memory(opcode, 0x0);
        cpu.run();

        // verify result
        assert_eq!(cpu.registers[0], 12, "failed to correctly add a constant and a register; a: {}, b: {}, result: {}", val_1, val_2, cpu.registers[0]);
    }
}
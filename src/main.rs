mod cpu;

use crate::cpu::CPU;

fn main() {
    let mut cpu: CPU = CPU::new();

    // load registers
    let register_values: [u8; 16] = [5, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    cpu.load_registers(&register_values);

    // load opcodes
    let opcodes: Vec<u16> = vec!(0x8014, 0x8014);
    cpu.load_opcodes_into_memory(&opcodes, 0x0);

    // run emulator
    cpu.run();
    cpu.print_debug_info();
}
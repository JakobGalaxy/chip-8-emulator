pub mod stack;
pub mod screen;
pub mod screen_ui;
mod cpu;

use crate::cpu::CPU;
use crate::screen::Screen;
use crate::screen_ui::SDLScreenUI;

fn main() {
    let sdl_context = sdl2::init().unwrap();

    let mut screen_ui = SDLScreenUI::new(25);
    screen_ui.init(sdl_context);
    screen_ui.clear();

    let screen = Screen::new(screen_ui);
    let mut cpu: CPU = CPU::new(screen, true, true, false);


    // load registers
    let register_values: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    cpu.load_registers(&register_values);

    // load opcodes
    let opcodes: Vec<u16> = vec!(0xA300, 0xD017, 0xD027, 0x7001, 0x1202);
    let ibm_opcodes: Vec<u16> = vec!(0x00e0, // clear screen
                                     0xa22a, // preparing to print I
                                     0x600c,
                                     0x6108,
                                     0xd01f, // printing I
                                     0x7009, // move x 9 pixels to the right
                                     0xa239, // prepare to print B (part 1)
                                     0xd01f, // print B (part 1)
                                     0xa248,
                                     0x7008,
                                     0xd01f,
                                     0x7004,
                                     0xa257,
                                     0xd01f,
                                     0x7008,
                                     0xa266,
                                     0xd01f,
                                     0x7008,
                                     0xa275,
                                     0xd01f,
                                     0x1228,
                                     0xff00, // start of I
                                     0xff00,
                                     0x3c00,
                                     0x3c00,
                                     0x3c00,
                                     0x3c00,
                                     0xff00,
                                     0xffff, // end of I (0xff * ff) -> start of B (part 1)
                                     0x00ff,
                                     0x0038,
                                     0x003f,
                                     0x003f,
                                     0x0038,
                                     0x00ff,
                                     0x00ff, // end of B (part 1)
                                     0x8000,
                                     0xe000,
                                     0xe000,
                                     0x8000,
                                     0x8000,
                                     0xe000,
                                     0xe000,
                                     0x80f8,
                                     0x00fc,
                                     0x003e,
                                     0x003f,
                                     0x003b,
                                     0x0039,
                                     0x00f8,
                                     0x00f8,
                                     0x0300,
                                     0x0700,
                                     0x0f00,
                                     0xbf00,
                                     0xfb00,
                                     0xf300,
                                     0xe300,
                                     0x43e0,
                                     0x00e0,
                                     0x0080,
                                     0x0080,
                                     0x0080,
                                     0x0080,
                                     0x00e0,
                                     0x00e0);
    cpu.load_opcodes_into_memory(&ibm_opcodes, 0x200);
    // cpu.load_opcodes_into_memory(&opcodes, 0x200);

    // load sprites
    let sprite: Vec<u8> = vec!(0b11111111,
                               0b10101001,
                               0b10100000,
                               0b10000001,
                               0b10100000,
                               0b10101001,
                               0b11111111);
    // cpu.load_bytes_into_memory(&sprite, 0x300);

    // run emulator
    cpu.run();
    cpu.print_debug_info();

    loop {}
}
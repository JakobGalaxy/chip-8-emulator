extern crate core;

pub mod stack;
pub mod screen;
mod chip8;
mod keypad;

use std::time::{Duration, Instant};
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use sdl2::pixels::Color;
use sdl2::render::{SdlError, WindowCanvas};
use sdl2::{EventPump, init, Sdl};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use chip8::Chip8;
use crate::keypad::Keypad;

const FPS: u64 = 60;

fn main() -> Result<(), ApplicationError> {
    let mut chip8 = Chip8::new(true, true, false);

    // let ibm_opcodes: Vec<u16> = vec!(0x00e0, // clear screen
    //                                  0xa22a, // preparing to print I
    //                                  0x600c,
    //                                  0x6108,
    //                                  0xd01f, // printing I
    //                                  0x7009, // move x 9 pixels to the right
    //                                  0xa239, // prepare to print B (part 1)
    //                                  0xd01f, // print B (part 1)
    //                                  0xa248,
    //                                  0x7008,
    //                                  0xd01f,
    //                                  0x7004,
    //                                  0xa257,
    //                                  0xd01f,
    //                                  0x7008,
    //                                  0xa266,
    //                                  0xd01f,
    //                                  0x7008,
    //                                  0xa275,
    //                                  0xd01f,
    //                                  0x1228,
    //                                  0xff00, // start of I
    //                                  0xff00,
    //                                  0x3c00,
    //                                  0x3c00,
    //                                  0x3c00,
    //                                  0x3c00,
    //                                  0xff00,
    //                                  0xffff, // end of I (0xff * ff) -> start of B (part 1)
    //                                  0x00ff,
    //                                  0x0038,
    //                                  0x003f,
    //                                  0x003f,
    //                                  0x0038,
    //                                  0x00ff,
    //                                  0x00ff, // end of B (part 1)
    //                                  0x8000,
    //                                  0xe000,
    //                                  0xe000,
    //                                  0x8000,
    //                                  0x8000,
    //                                  0xe000,
    //                                  0xe000,
    //                                  0x80f8,
    //                                  0x00fc,
    //                                  0x003e,
    //                                  0x003f,
    //                                  0x003b,
    //                                  0x0039,
    //                                  0x00f8,
    //                                  0x00f8,
    //                                  0x0300,
    //                                  0x0700,
    //                                  0x0f00,
    //                                  0xbf00,
    //                                  0xfb00,
    //                                  0xf300,
    //                                  0xe300,
    //                                  0x43e0,
    //                                  0x00e0,
    //                                  0x0080,
    //                                  0x0080,
    //                                  0x0080,
    //                                  0x0080,
    //                                  0x00e0,
    //                                  0x00e0);
    // chip8.load_opcodes_into_memory(&ibm_opcodes, 0x200);

    let sound_opcodes: Vec<u16> = vec!( 0x613C, // set V1 to 60
                                        0x6202, // set V2 to 1
                                        0x631E, // set V3 to 30
                                        0xF318, // set sound timer to V3
                                        0xF115, // set delay timer to V1
                                        0xF007, // loop: set VX to delay timer
                                        0x3000, // check if V0 == 0
                                        0x120A, // if not -> jump back to loop:
                                        0x8125, // decrement V1 by V2
                                        0x411E, // check if V1 == 30
                                        0x613C, // if yes -> set V1 to 60
                                        0x1206, // if yes -> repeat program
                                        );
    chip8.load_opcodes_into_memory(&sound_opcodes, 0x200);

    run(&mut chip8, 20)?;

    return Ok(());
}

#[derive(Debug)]
enum ApplicationError {
    Sdl(String),
    Chip8(chip8::Chip8Error),
}

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

fn init_audio_device(sdl_context: &Sdl) -> Result<AudioDevice<SquareWave>, ApplicationError> {
    let audio_subsystem = sdl_context.audio().map_err(|err| ApplicationError::Sdl(err))?;

    let audio_device_spec = AudioSpecDesired {
        freq: Some(44_100),
        channels: Some(1), // mono
        samples: None,
    };

    let audio_device = audio_subsystem.open_playback(None, &audio_device_spec, |spec| {
        println!("audio spec: {:?}", spec);

        SquareWave {
            phase_inc: 440.0 / spec.freq as f32,
            phase: 0.0,
            volume: 0.05,
        }
    }).map_err(|err| ApplicationError::Sdl(err))?;

    return Ok(audio_device);
}

fn init_canvas(sdl_context: &Sdl, screen_scale: u32) -> Result<WindowCanvas, ApplicationError> {
    let video_subsystem = sdl_context.video().map_err(|err| ApplicationError::Sdl(err))?;

    let window = video_subsystem
        .window("CHIP-8 emulator", screen::WIDTH * screen_scale, screen::HEIGHT * screen_scale)
        .position_centered()
        .build()
        .map_err(|err| ApplicationError::Sdl(err.to_string()))?;

    let mut canvas = window
        .into_canvas()
        .build()
        .map_err(|err| ApplicationError::Sdl(err.to_string()))?;

    canvas.set_draw_color(Color::BLACK);
    canvas.clear();
    canvas.present();

    return Ok(canvas);
}

fn init_event_pump(sdl_context: &Sdl) -> Result<EventPump, ApplicationError> {
    let event_pump = sdl_context.event_pump().map_err(|err| ApplicationError::Sdl(err))?;

    return Ok(event_pump);
}

fn get_input(event_pump: &mut EventPump) -> Result<Keypad, ()> {
    // original keypad
    // 1 2 3 C
    // 4 5 6 D
    // 7 8 9 E
    // A 0 B F

    // mapping
    // 1 2 3 4
    // Q W E R
    // A S D F
    // Z X C V (Z can also be Y)

    let mut keypad = Keypad::new();

    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => return Err(()),
            Event::KeyDown {
                keycode: Some(keycode),
                ..
            } => {
                match keycode {
                    Keycode::Num1 => keypad.set_key(0x1),
                    Keycode::Num2 => keypad.set_key(0x2),
                    Keycode::Num3 => keypad.set_key(0x3),
                    Keycode::Num4 => keypad.set_key(0xC),
                    Keycode::Q => keypad.set_key(0x4),
                    Keycode::W => keypad.set_key(0x5),
                    Keycode::E => keypad.set_key(0x6),
                    Keycode::R => keypad.set_key(0xD),
                    Keycode::A => keypad.set_key(0x7),
                    Keycode::S => keypad.set_key(0x8),
                    Keycode::D => keypad.set_key(0x9),
                    Keycode::F => keypad.set_key(0xE),
                    Keycode::Z | Keycode::Y => keypad.set_key(0xA),
                    Keycode::X => keypad.set_key(0x0),
                    Keycode::C => keypad.set_key(0xB),
                    Keycode::V => keypad.set_key(0xF),
                    _ => {}
                }
            }
            _ => {}
        }
    }

    return Ok(keypad);
}

fn update_audio_device(audio_device: &AudioDevice<SquareWave>, chip8: &Chip8) {
    if chip8.playing_sound() {
        audio_device.resume();
    } else {
        audio_device.pause();
    }
}

fn update_screen(canvas: &mut WindowCanvas, chip8: &Chip8, screen_scale: u32) {
    let frame_buffer = chip8.get_frame_buffer();

    for (y_pos, row) in frame_buffer.iter().enumerate() {
        for (x_pos, pixel_val) in row.iter().enumerate() {
            let color = if *pixel_val { Color::WHITE } else { Color::BLACK };

            let real_x_pos = (x_pos as u32) * screen_scale;
            let real_y_pos = (y_pos as u32) * screen_scale;

            let rect = Rect::new(real_x_pos as i32, real_y_pos as i32, screen_scale, screen_scale);

            canvas.set_draw_color(color);
            canvas.fill_rect(rect).unwrap();
        }
    }

    canvas.present();
}

fn run(chip8: &mut Chip8, screen_scale: u32) -> Result<(), ApplicationError> {
    let sdl_context = sdl2::init().map_err(|err| ApplicationError::Sdl(err))?;

    let audio_device = init_audio_device(&sdl_context)?;
    let mut canvas = init_canvas(&sdl_context, screen_scale)?;
    let mut event_pump = init_event_pump(&sdl_context)?;

    let frame_duration = Duration::from_nanos(1_000_000_000 / FPS);
    let mut last_frame_timestamp = Instant::now();

    loop {
        // get input and load keypad
        if let Ok(keypad) = get_input(&mut event_pump) {
            chip8.load_keypad(keypad);
        } else {
            break;
        }

        // run emulator
        chip8.run_frame(frame_duration).map_err(|err| ApplicationError::Chip8(err))?;

        // update audio device
        update_audio_device(&audio_device, &chip8);

        // update screen
        update_screen(&mut canvas, &chip8, screen_scale);

        // wait for frame duration to pass
        let sleep_duration = frame_duration.checked_sub(last_frame_timestamp.elapsed()).unwrap_or(Duration::new(0, 0));
        std::thread::sleep(sleep_duration);
        last_frame_timestamp = Instant::now();
    }

    return Ok(());
}
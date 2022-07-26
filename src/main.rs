extern crate core;

pub mod stack;
pub mod screen;
mod config;
mod chip8;
mod keypad;

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::time::{Duration, Instant};
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use sdl2::pixels::Color;
use sdl2::render::{WindowCanvas};
use sdl2::{EventPump, Sdl};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use confy;
use chip8::Chip8;
use keypad::Keypad;
use crate::config::ApplicationConfig;

// GUI constants
const FPS: u64 = 60;

fn main() -> Result<(), ApplicationError> {

    // load config
    let config = config::load_config().map_err(|err| ApplicationError::Config(err))?;

    let mut chip8 = Chip8::new(true, true, false);

    // load fonts data
    let font_data: Vec<u8> = load_binary_file(&config.font_path)?;
    chip8.load_font(&font_data).map_err(|err| ApplicationError::Chip8(err))?;

    // load program
    let program_data: Vec<u8> = load_binary_file(&config.program_path)?;
    chip8.load_program(&program_data).map_err(|err| ApplicationError::Chip8(err))?;

    run(&mut chip8, config)?;

    return Ok(());
}

fn load_binary_file(path: &str) -> Result<Vec<u8>, ApplicationError> {
    let mut file = File::open(Path::new(path)).map_err(|err| ApplicationError::IO(err))?;

    let mut data: Vec<u8> = vec!();

    file.read_to_end(&mut data).map_err(|err| ApplicationError::IO(err))?;

    return Ok(data);
}

#[derive(Debug)]
enum ApplicationError {
    Sdl(String),
    Chip8(chip8::Chip8Error),
    Config(confy::ConfyError),
    IO(io::Error),
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

fn get_input(event_pump: &mut EventPump, keypad: &mut Keypad) -> Result<(), ()> {
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
            },
            Event::KeyUp {
                keycode: Some(keycode),
                ..
            } => {
                match keycode {
                    Keycode::Num1 => keypad.unset_key(0x1),
                    Keycode::Num2 => keypad.unset_key(0x2),
                    Keycode::Num3 => keypad.unset_key(0x3),
                    Keycode::Num4 => keypad.unset_key(0xC),
                    Keycode::Q => keypad.unset_key(0x4),
                    Keycode::W => keypad.unset_key(0x5),
                    Keycode::E => keypad.unset_key(0x6),
                    Keycode::R => keypad.unset_key(0xD),
                    Keycode::A => keypad.unset_key(0x7),
                    Keycode::S => keypad.unset_key(0x8),
                    Keycode::D => keypad.unset_key(0x9),
                    Keycode::F => keypad.unset_key(0xE),
                    Keycode::Z | Keycode::Y => keypad.unset_key(0xA),
                    Keycode::X => keypad.unset_key(0x0),
                    Keycode::C => keypad.unset_key(0xB),
                    Keycode::V => keypad.unset_key(0xF),
                    _ => {}
                }
            },
            _ => {},
        }
    }

    return Ok(());
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

fn run(chip8: &mut Chip8, config: ApplicationConfig) -> Result<(), ApplicationError> {
    let sdl_context = sdl2::init().map_err(|err| ApplicationError::Sdl(err))?;

    let audio_device = init_audio_device(&sdl_context)?;
    let mut canvas = init_canvas(&sdl_context, config.screen_scale)?;
    let mut event_pump = init_event_pump(&sdl_context)?;

    let frame_duration = Duration::from_nanos(1_000_000_000 / FPS);
    let mut last_frame_timestamp = Instant::now();

    let mut keypad = Keypad::new();

    loop {
        // check if program has finished
        if chip8.reached_end_of_file() {
            break;
        }

        // get input and load keypad
        if let Ok(_) = get_input(&mut event_pump, &mut keypad) {
            chip8.load_keypad(&keypad);
        } else {
            break;
        }

        // run emulator
        chip8.run_frame(frame_duration).map_err(|err| ApplicationError::Chip8(err))?;

        // update audio device
        update_audio_device(&audio_device, &chip8);

        // update screen
        update_screen(&mut canvas, &chip8, config.screen_scale);

        // wait for frame duration to pass
        let sleep_duration = frame_duration.checked_sub(last_frame_timestamp.elapsed()).unwrap_or(Duration::new(0, 0));
        std::thread::sleep(sleep_duration);
        last_frame_timestamp = Instant::now();
    }

    return Ok(());
}
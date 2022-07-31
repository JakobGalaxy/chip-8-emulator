extern crate sdl2;

use std::cmp;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::Sdl;
use crate::screen::{HEIGHT, WIDTH};

const INIT_FADE_OUT_COLOR_VAL: u8 = 250;
const FADE_OUT_DURATION: Duration = Duration::from_millis(200);

trait ScreenUI {
    fn new(size_multiplier: u32) -> Self;

    fn flip_pixel(&mut self, x_pos: u8, y_pos: u8, on: bool);
}

pub struct SDLScreenUI {
    height: u32,
    width: u32,
    size_multiplier: u32,
    canvas: Option<WindowCanvas>,
    /// holds the pixel coords that need to be faded out
    fade_map: HashMap<(u8, u8), u8>,

    last_update: Instant,
}

impl SDLScreenUI {
    pub fn new(size_multiplier: u32) -> Self {
        let height = (HEIGHT as u32) * size_multiplier;
        let width = (WIDTH as u32) * size_multiplier;

        return SDLScreenUI {
            height,
            width,
            size_multiplier,
            canvas: None,
            fade_map: HashMap::new(),
            last_update: Instant::now(),
        };
    }

    pub fn init(&mut self, sdl_context: Sdl) {
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem.window("CHIP-8 emulator", self.width, self.height)
            .position_centered()
            .build()
            .unwrap();
        let canvas = window.into_canvas().build().unwrap();

        self.canvas = Some(canvas);
    }

    pub fn flip_pixel(&mut self, x_pos: u8, y_pos: u8, on: bool) {
        let color = if on {
            // stop fade out if in process
            self.fade_map.remove(&(x_pos, y_pos));

            Color::WHITE
        } else {
            // start fade out
            self.fade_map.insert((x_pos, y_pos), INIT_FADE_OUT_COLOR_VAL);
            Color::RGB(INIT_FADE_OUT_COLOR_VAL, INIT_FADE_OUT_COLOR_VAL, INIT_FADE_OUT_COLOR_VAL)
        };

        if let Some(canvas) = &mut self.canvas {
            SDLScreenUI::draw_pixel(canvas, x_pos, y_pos, color, self.size_multiplier);
            canvas.present();
        }
    }

    fn draw_pixel(canvas: &mut WindowCanvas, x_pos: u8, y_pos: u8, color: Color, size_multiplier: u32) {
            let real_x_pos = (x_pos as u32) * size_multiplier;
            let real_y_pos = (y_pos as u32) * size_multiplier;

            let rect = Rect::new(real_x_pos as i32, real_y_pos as i32, size_multiplier, size_multiplier);

            canvas.set_draw_color(color);
            canvas.fill_rect(rect).unwrap();
    }

    pub fn clear(&mut self) {
        if let Some(canvas) = &mut self.canvas {
            canvas.set_draw_color(Color::BLACK);
            canvas.clear();
            canvas.present();
        }
    }

    pub fn update(&mut self) {
        self.fade_out();

        self.last_update = Instant::now();

        if let Some(canvas) = &mut self.canvas {
            canvas.present();
        }
    }

    fn fade_out(&mut self) {
        let time_elapsed = self.last_update.elapsed();
        let fade_out_fraction: f32 = time_elapsed.as_secs_f32() / FADE_OUT_DURATION.as_secs_f32();
        let fade_out_val: i32 = ((INIT_FADE_OUT_COLOR_VAL as f32) * fade_out_fraction) as i32;

        if let Some(canvas) = &mut self.canvas {
            for (key, val) in self.fade_map.iter_mut() {
                let (x_pos, y_pos) = key;
                *val = cmp::max((*val as i32) - fade_out_val, 0) as u8;
                let color = Color::RGB(*val, *val, *val);

                SDLScreenUI::draw_pixel(canvas, *x_pos, *y_pos, color, self.size_multiplier);
            }
        }
    }
}
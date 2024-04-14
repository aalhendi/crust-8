use sdl2::{pixels::Color, rect::Rect, render::Canvas, video::Window};

use crate::{
    vm::{SCREEN_HEIGHT, SCREEN_WIDTH},
    SCALE,
};

pub struct Screen {
    pixels: [[bool; SCREEN_WIDTH]; SCREEN_HEIGHT],
    draw_flag: bool,
    canvas: Canvas<Window>,
}

impl Screen {
    pub fn new(canvas: Canvas<Window>) -> Self {
        Self {
            pixels: [[false; SCREEN_WIDTH]; SCREEN_HEIGHT],
            draw_flag: true,
            canvas,
        }
    }

    pub fn clear(&mut self) {
        self.pixels = [[false; SCREEN_WIDTH]; SCREEN_HEIGHT];
        self.draw_flag = true;
    }

    pub fn draw(&mut self) -> Result<(), String> {
        if !self.draw_flag {
            return Ok(());
        }
        let mut pixel: u8;
        let pt = |p: usize| (p as i32) * (SCALE as i32);

        for y in 0..32 {
            for x in 0..64 {
                pixel = if self.pixels[y][x] { 255 } else { 0 };

                self.canvas.set_draw_color(Color::RGB(pixel, pixel, pixel));
                self.canvas
                    .fill_rect(Some(Rect::new(pt(x), pt(y), SCALE as u32, SCALE as u32)))?;
            }
        }

        self.canvas.present();
        self.draw_flag = false;
        Ok(())
    }

    pub fn set_draw_flag(&mut self, draw_flag: bool) {
        self.draw_flag = draw_flag;
    }

    pub fn get_pixel_state(&self, x: usize, y: usize) -> bool {
        self.pixels[y][x]
    }

    pub fn xor_pixel(&mut self, x: usize, y: usize, state: bool) {
        self.pixels[y][x] ^= state
    }

}

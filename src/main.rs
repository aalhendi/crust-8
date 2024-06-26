mod vm;
mod display;
use std::fs;

use sdl2::{event::Event, keyboard::Keycode, pixels::Color, render::Canvas, video::Window};
use vm::{SCREEN_HEIGHT, SCREEN_WIDTH, VM};

fn setup(canvas: Canvas<Window>) -> VM {
    let file =
        fs::read("./chip8-roms/games/Pong [Paul Vervalin, 1990].ch8").expect("Unable to read file");
    let mut vm = VM::new(canvas);
    vm.load_rom(&file);
    vm
}

const SCALE: usize = 15;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window(
            "Crust-8",
            (SCREEN_WIDTH * SCALE) as u32,
            (SCREEN_HEIGHT * SCALE) as u32,
        )
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump()?;

    let mut vm = setup(canvas);

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Some(k) = keycode_to_input(keycode) {
                        vm.set_key(k, true);
                    }
                }

                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Some(k) = keycode_to_input(keycode) {
                        vm.set_key(k, false);
                    }
                }
                _ => {}
            }
        }

        vm.decode();
        vm.tick_timers();
        vm.display.draw()?;

        // TODO(aalhendi): Tickrate
        std::thread::sleep(std::time::Duration::from_millis(2));
    }

    Ok(())
}

fn keycode_to_input(key: Keycode) -> Option<usize> {
    Some(match key {
        Keycode::Num1 => 0x1,
        Keycode::Num2 => 0x2,
        Keycode::Num3 => 0x3,
        Keycode::Num4 => 0xC,
        Keycode::Q => 0x4,
        Keycode::W => 0x5,
        Keycode::E => 0x6,
        Keycode::R => 0xD,
        Keycode::A => 0x7,
        Keycode::S => 0x8,
        Keycode::D => 0x9,
        Keycode::F => 0xE,
        Keycode::Z => 0xA,
        Keycode::X => 0x0,
        Keycode::C => 0xB,
        Keycode::V => 0xF,
        _ => return None,
    })
}

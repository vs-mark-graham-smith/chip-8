extern crate sdl2;

mod display;
mod keypad;
mod cpu;
use sdl2::keyboard::Keycode;

use crate::cpu::Cpu;
use sdl2::event::Event;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cpu = Cpu::new();

    cpu.load_game("pong.ch8");
    // cpu.load_game("test_opcode.ch8");

    'main : loop {
        for event in cpu.display.sdl.event_pump().unwrap().poll_iter() {
            match event {
                Event::KeyUp { keycode, .. } => match keycode {
                    Some(key) => cpu.keypad.press(key, false),
                    None => {}
                },
                Event::KeyDown { keycode, .. } => match keycode {
                    Some(key) => cpu.keypad.press(key, true),
                    None => {}
                },
                Event::Quit {..} => break 'main,
                _ => {}
            }
        }

        // 'key : loop {
        //     for event in cpu.display.sdl.event_pump().unwrap().poll_iter() {
        //         match event {
        //             Event::KeyDown {keycode, ..} => match keycode {
        //                 Some(key) => match key {
        //                     Keycode::Up => break 'key,
        //                     Keycode::Down => {

        //                     }
        //                     _ => {}
        //                 }
        //                 _ => {}
        //             },
        //             Event::Quit {..} => break 'main,
        //             _ => {}
        //         }
        //     }
        // }

        cpu.cpu_cycle();
        cpu.display.draw_screen();
    }

    Ok(())
}

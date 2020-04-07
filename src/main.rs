///////////////////////////////////////////////////////////////////////////////
// Project description
// ¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯
// Name: myChip8
//
// Author: Laurence Muller
// Contact: laurence.muller@gmail.com
//
// License: GNU General Public License (GPL) v2
// ( http://www.gnu.org/licenses/old-licenses/gpl-2.0.html )
//
// Copyright (C) 2011 Laurence Muller / www.multigesture.net
///////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////
// Rust port
// ¯¯¯¯¯¯¯¯¯
// Name: dale8
//
// Author: Daniel Pistelli
//
// License: GNU General Public License (GPL) v2
// ( http://www.gnu.org/licenses/old-licenses/gpl-2.0.html )
//
// Copyright (C) 2019 Daniel Pistelli / ntcore.com
///////////////////////////////////////////////////////////////////////////////

extern crate sdl2;

use std::path::Path;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::audio::{AudioCallback, AudioSpecDesired, AudioSpecWAV, AudioCVT};

mod cpu;
use std::env;

const SCREEN_WIDTH: u32 = 64;
const SCREEN_HEIGHT: u32 = 32;

const DISPLAY_MODIFIER: u32 = 10;

const DISPLAY_WIDTH: u32 = SCREEN_WIDTH * DISPLAY_MODIFIER;
const DISPLAY_HEIGHT: u32 = SCREEN_HEIGHT * DISPLAY_MODIFIER;

struct Sound {
    data: Vec<u8>,
    volume: f32,
    pos: usize,
}

impl AudioCallback for Sound {
    type Channel = u8;

    fn callback(&mut self, out: &mut [u8]) {
        for dst in out.iter_mut() {
            *dst = (*self.data.get(self.pos).unwrap_or(&0) as f32 * self.volume) as u8;
            self.pos += 1;
        }
    }
}

fn main()
{
    let args: Vec<String> = env::args().collect();
    if args.len() != 2
    {
        println!("syntax: rusty-8 [rom_file]");
        return;
    }
    let mut vm = cpu::VM::new();
    if !vm.load_application(&args[1])
    {
        println!("failed to load rom");
        return
    }

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let window = video_subsystem.window("rusty-8", DISPLAY_WIDTH, DISPLAY_HEIGHT).position_centered().build()
        .map_err(|e| e.to_string()).unwrap();

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string()).unwrap();
    let texture_creator = canvas.texture_creator();

    let mut texture  = texture_creator.create_texture_streaming(PixelFormatEnum::RGB24, SCREEN_WIDTH,
        SCREEN_HEIGHT).map_err(|e| e.to_string()).unwrap();

    // TODO: bring er back in
    // let mut _audio_device = None;
    let has_sound = Path::new("beep.wav").exists();

    let mut timer = 0;

    'mainloop: loop
    {
        for event in sdl_context.event_pump().unwrap().poll_iter()
        {
            match event
            {
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } |
                Event::Quit { .. } => break 'mainloop,

                // keydown

                _ => {}
            }
        }

        if timer == 2000
        {
            vm.emulate_cycle();
            timer = 0;
        }
        else
        {
            timer += 1;
        }

        if vm.draw_flag
        {
            texture.with_lock(None, |buffer: &mut [u8], pitch: usize|
            {
                for y in 0..SCREEN_HEIGHT as usize
                {
                    for x in 0..SCREEN_WIDTH as usize
                    {
                        let offset: usize = y*pitch + x*3;
                        let mut color: u8 = 0;
                        if vm.gfx[((y * SCREEN_WIDTH as usize) + x) as usize] != 0
                        {
                            color = 255;
                        }
                        buffer[offset] = color;
                        buffer[offset + 1] = color;
                        buffer[offset + 2] = color;
                    }
                }
            }).unwrap();

            canvas.clear();
            canvas.copy(&texture, None, Some(Rect::new(0, 0, DISPLAY_WIDTH, DISPLAY_HEIGHT))).unwrap();
            canvas.present();

            vm.draw_flag = false;
        }
    }
}

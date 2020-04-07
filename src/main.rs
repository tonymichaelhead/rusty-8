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
}

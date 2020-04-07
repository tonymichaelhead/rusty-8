use std::fs::File;
use std::io::prelude::*;
use rand;


const FONTSET: [u8; 80] =
[
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct VM
{
    pc: u16,
    opcode: u16,
    ir: u16,
    sp: u16,

    v: [u8; 16],
    stack: [u16; 16],
    memory: [u8; 4096],

    pub gfx: [u8; 2048],
    pub key: [u8; 16],

    delay_timer: u8,
    sound_timer: u8,

    pub draw_flag: bool,
    pub beep_flag: bool,
}

impl VM
{
    pub fn new() -> VM
    {
        let mut vm = VM
        {
            pc: 0x200, // Program counter starts at 0x200
            opcode: 0, // Reset current opcode
            ir: 0, // Reset index register
            sp: 0, // Reset stack pointer

            // CPU registers = 15 8-bit general purpose registers name V0, V1, up to VE.
            // the 16th is for the 'carry flag'
            v: [0; 16],
            stack: [0; 16],
            // Chip 8 has 4K memory
            memory: [0; 4096]

            gfx: [0; 2048], // 2048 pixels (64 x 32)
            key: [0; 16],

            delay_timer: 0,
            sound_timer: 0,

            draw_flag: true,
            beep_flag: false,
        };

        // Load fontset
        for i in 0..80
        {
            vm.memory[i] = FONTSET[i];
        }

        return vm;
    }

    pub fn emulate_cycle(& mut self)
    {
        // fetch opcode
        self.opcode = (self.memory[self.pc as usize] as u16) << 8 | (self.memory[(self.pc + 1) as usize] as u16);

        println!("opcode: {:02X}{:02X}", (self.opcode >> 8) as u8, self.opcode as u8);

        // process opcode
        match self.opcode & 0xF000
        {
            0x0000 =>
            {
                match self.opcode & 0x000F
                {
                    0x0000 => // 0x00E0: clears the screen
                    {
                        for i in 0..2048
                        {
                            self.gfx[i] = 0;
                        }
                        self.draw_flag = true;
                        self.pc += 2;
                    },
                    0x000E => // 0x00EE: returns from subroutine
                    {
                        self.sp -= 1;                           // 16 levels of stack, decrease stack pointer to prevent overwrite
                        self.pc = self.stack[self.sp as usize]; // put the stored return address from the stack back into the program counter
                        self.pc += 2                            // don't forget to increase the program counter!
                    }
                    _ =>
                    {
                        panic!("unknown opcode [0x0000]: 0x{:X}.", self.opcode);
                    },
                }
            },

            0x1000 => // 0x1NNN: jumps to address NNN
            {
                self.pc = self.opcode & 0x0FFF;
            },

            0x2000 => // 0x2NNN: calls subroutine at NNN
            {
                self.stack[self.sp as usize] = self.pc; // store your current address in the stack
                self.sp += 1;                           // increment stack pointer
                self.pc = self.opcode & 0x0FFF;         // set the program counter to the address at NNN
            },

            0x3000 => // 0x3XNN: skips the next instruction if VX equals NN
            {
                if self.v[((self.opcode & 0x0F00) >> 8) as usize] == (self.opcode & 0x0FF) as u8
                {
                    self.pc += 4;
                }
                else
                {
                    self.pc += 2;
                }
            },

            0x4000 => // 0x4XNN: skips the next instruction if VX DOESN'T equal NN
            {
                if self.v[((self.opcode & 0x0F00) >> 8) as usize] != (self.opcode & 0x00FF) as u8
                {
                    self.pc += 4;
                }
                else
                {
                    self.pc += 2;
                }
            },

            0x5000 => // 0x5XY0: skips the next intstruction if VX equals VY
            {
                if self.v[((self.opcode & 0x0F00) >> 8) as usize] == self.v[((self.opcode & 0x00F0) >> 4) as usize]
                {
                    self.pc += 4;
                }
                else
                {
                    self.pc += 2;
                }
            },

            0x6000 => // 0x6XNN: sets VX to NN
            {
                self.v[((self.opcode & 0x0F00) >> 8) as usize] = (self.opcode & 0x00FF) as u8;
                self.pc += 2;
            },

            0x7000 => // 0x7XNN: adds NN to VX
            {
                let pos: usize = ((self.opcode & 0x0F00) >> 8) as usize;
                self.v[pos] = self.v[pos].wrapping_add((self.opcode & 0x00FF) as u8);
                self.pc += 2;
            },

            0x8000 =>
            {
                match self.opcode & 0x000F
                {
                    0x0000 => // 0x8XY0: sets VX to the value of VY
                    {
                        self.v[((self.opcode & 0x0F00) >> 8) as usize] = self.v[((self.opcode & 0x00F0) >> 4) as usize];
                        self.pc += 2;
                    },

                    0x0001 => // 0x8XY1 sets VX to "VX OR VY"
                    {
                        self.v[((self.opcode & 0x0F00) >> 8) as usize]  |= self.v[((self.opcode & 0x00F0) >> 4) as usize];
                        self.pc += 2;
                    },
                    0x0002 => // 0x8XY2: sets VX to "VX AND VY"
                    {
                        self.v[((self.opcode & 0x0F00) >> 8) as usize]  &= self.v[((self.opcode & 0x00F0) >> 4) as usize];
                        self.pc += 2;
                    },
                    0x0003 => // 0x8XY3: sets VX to "VX XOR VY"
                    {
                        self.v[((self.opcode & 0x0F00) >> 8) as usize]  ^= self.v[((self.opcode & 0x00F0) >> 4) as usize];
                        self.pc += 2;
                    },
                    0x0004 => // 0x8XY4: adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there isn't
                    {
                        if self.v[((self.opcode & 0x00f0) >> 4) as usize] > (0xFF - self.v[((self.opcode & 0x0F00) >> 8) as usize])
                        {
                            self.v[0xF] = 1; // carry
                        }
                        else
                        {
                            self.v[0xF] = 0;
                        }
                        let pos: usize = ((self.opcode & 0x0F00) >> 8) as usize;
                        self.v[pos] = self.v[pos].wrapping_add(self.v[((self.opcode & 0x00F0) >> 4) as usize]);
                        self.pc += 2;
                    },
                    0x0005 => // 0x8XY5: VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there isn't
                    {
                        let pos: usize = ((self.opcode & 0x0F00) >> 8) as usize;
                        if self.v[((self.opcode & 0x00F0) >> 4) as usize] > self.v[pos]
                        {
                            self.v[0xF] = 0; // there is a borrow
                        }
                        else
                        {
                            self.v[0xF] = 1;
                        }
                        self.v[pos] = self.v[pos].wrapping_sub(self.v[((self.opcode & 0x00F0) >> 4) as usize]);
                        self.pc += 2;
                    },
                    0x0006 =>
                    {
                    },
                    0x0007 =>
                    {
                    },
                    0x000E =>
                    {
                    },
                }
            },

            0x9000 =>
            {
            },

            0xA000 => // ANNN: sets I to the address NNN
            {
                self.ir = self.opcode & 0x0FFF;
                self.pc += 2;
            },

            0xB000 =>
            {
            },

            0xC000 =>
            {
            },

            // DXYN: draws a sprite at coordinate (VX,VY) that has a width of 8 pixels and a height of N pixels.
            // each row of 8 pixels is read as bit-coded starting from memory location ri.
            // ri value doesn't change after the executioon of this instruction.
            // VF is set to 1 if any screen pixels are flipped from set to unset when the sprite is
            // drawn, and to 0 if that dosen't happen
            0xD000 =>
            {
                // Fetch the position and height of the sprite based on opcode
                let x = self.v[((self.opcode & 0x0F00) >> 8) as usize] as u16;
                let y = self.v[((self.opcode & 0x00F0) >> 4) as usize] as u16;
                let height = self.opcode & 0x000F;

                // reset register VF (but what is that?)
                self.v[0xF] = 0;
                // loop over each row?
                for yline in 0..height
                {
                    // fetch the pixel value from the memory starting at location I (ir)
                    let pixel = self.memory[(self.ir + yline) as usize] as u16;
                    // loop over 8 bits in one row
                    for xline in 0..8
                    {
                        // check  if the current evaluated pixel is set to 1 (note 0x80 >> xline
                        // scan through the byte, one bit at a time)
                        if (pixel & (0x80 >> xline)) != 0
                        {
                            let pos = (x + xline + ((y + yline) * 64)) as usize;
                            if pos < 2048
                            {
                                if self.gfx[pos] ==1
                                {
                                    // Check if the pixel on the display is set to one. if it is, we need
                                    // to register the collision by setting the VF register
                                    self.v[0xF] = 1;
                                }
                                // set the pixel value by using XOR
                                self.gfx[pos] ^= 1;
                            }
                        }
                    }
                }
                // Since we've changed our gfx[] array, we'll want to update the screen
                self.draw_flag = true;
                // update the program counter to move to the next opcode (2 bytes ahead)
                self.pc += 2;
            },

            0xE000 => {
                match self.opcode & 0x00FF
                {
                    0x009E => // EX9E: skips the next instruction if the key stored in VX is pressed
                    {
                        if self.key[self.v{((self.opcode & 0x0F00) >> 8) as usize} as usize] != 0
                        {
                            self.pc += 4;
                        }
                        else
                        {
                            self.pc += 2;
                        }
                    },

                    0x00A1 => // EXA1: skips the next instructions if the key stored in VX isn't pressed
                    {
                        if self.key[self.v[((self.opcode & 0x0F00) >> 8) as usize] usize] == 0
                        {
                            self.pc += 4;
                        }
                        else
                        {
                            self.pc += 2;
                        }
                    },

                    _ =>
                    {
                        panic!("unknown opcode [0xE000]: 0x{:X}.", self.opcode);
                    },
                }

            },

            0xF000 =>
            {
                match self.opcode & 0x00FF
                {

                    0x00007 =>
                    {
                    },
                    0x0000A =>
                    {
                    },
                    0x00015 =>
                    {
                    },
                    0x00018 =>
                    {
                    },
                    0x0001E =>
                    {
                    },
                    0x00029 =>
                    {
                    },
                    0x00033 => // FX33: stores the binary-coded decimal representation of VX at the addresses ir, ir plus 1, and ir plus 2
                    {
                        self.memory[self.ir as usize] = self.v[((self.opcode & 0x0F00) >> 8) as usize] / 100;
                        self.memory[(self.ir + 1) as usize] = (self.v[((self.opcode & 0x0F00) >> 8) as usize] / 10) % 10;
                        self.memory[(self.ir + 2) as usize] = (self.v[((self.opcode & 0x0F00) >> 8) as usize] % 100) % 10;
                        self.pc += 2;
                    },
                    0x00055 =>
                    {
                    },
                    0x00065 =>
                    {
                    },

                    _ =>
                    {
                        panic!("unknown opcode [0xF000]: 0x{:X}.", self.opcode);
                    },
                }
            },

            _ =>
            {
                panic!("unknown opcode [0x0000]: 0x{:X}.", self.opcode);
            },
        }

        // update timers
        if self.delay_timer > 0
        {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1
            {
                self.beep_flag = true;
            }
            self.sound_timer -= 1;
        }
    }

    pub fn load_application(& mut self, filename: &str) -> bool
    {
        // open the file
        let mut file = File::open(filename).expect("file error");

        // get the file size
        let fsize = file.metadata().unwrap().len();

        // read the file to a buffer
        let mut buffer = vec![];
        file.read_to_end(&mut buffer).expect("couldn't read file");
        drop(file);

        // copy the buffer to the chip8 memory
        if (4096 - 512) > fsize
        {
            for i in 0..fsize
            {
                self.memory[(i + 512) as usize] = buffer[i as usize];
            }
            else
            {
                panic!("ROM too big for memory");
            }

            return true;
        }
    }
}










fn main() {
    println!("Hello, world!");
}

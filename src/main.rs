use std::fs::File;

pub struct VM
{
    pc: u16,
    opcode: u16,
    ir: u16,
    sp: u16,

    memory: [u8; 4096],
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
        }

        // Clear display
        // Clear stack
        // Clear registers V0-VF
        // Clear memory
        //
        // Load fontset
        for i in 0..80
        {
            vm.memory[i] = FONTSET[i];
        }

        // Reset timers

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
            // some opcodes
            0x0000 =>
            {
                match self.opcode & 0x000F
                {
                    0x0000 => // 0x00E0: clears the screen
                    {
                        // execute opcode


                    },
                    0x000E => // 0x00EE: returns from subroutine
                    {
                        // execute opcode
                    }
                    _ =>
                    {
                        panic!("unknown opcode [0x0000]: 0x{:X}.", self.opcode);
                    },
                }
            },

            0x2000 => // 0x2NNN: calls subroutine at NNN.
            {
                self.stack[self.sp as usize] = self.pc; // store current address in stack
                self.sp +=                              // increment stack pointer
                self.pc = self.opcode & 0x0FFF;
            },
            0x8000 =>
            {
                match self.opcode & 0x000F
                {
                    0x0000 =>
                    {
                    },
                    0x0001 =>
                    {
                    },
                    0x0002 =>
                    {
                    },
                    0x0003 =>
                    {
                    },
                    0x0004 =>
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
                    0x0005 =>
                    {
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

            0xA000 => // ANNN: sets I to the address NNN
            {
                self.ir = self.opcode & 0x0FFF;
                self.pc += 2;
            },

            // more opcodes

            _ =>
            {
                panic!("unknown opcode [0xF000]: 0x{:X}.", self.opcode);
            },

            // DXYN: draws a sprite at coordinate (VX,VY) that has a width of 8 pixels and a height of N pixels.
            // each row of 8 pixels is read as bit-coded starting from memory location ri.
            // ri value doesn't change after the executioon of this instruction.
            // VF is set to 1 if any screen pixels are flipped from set to unset when the sprite is
            // drawn, and to 0 if that dosen't happen
            0xD000 =>
            {
                // TODO: Add in step by step comments from tutorial
                let x = self.v[((self.opcode & 0x0F00) >> 8) as usize] as u16;
                let y = self.v[((self.opcode & 0x00F0) >> 4) as usize] as u16;
                let height = self.opcode & 0x000F;

                self.v[0xF] = 0;
                for yline in 0..height
                {
                    let pixel = self.memory[(self.ir + yline) as usize] as u16;
                    for xline in 0..8
                    {
                        if (pixel & (0x80 >> xline)) != 0
                        {
                            let pos = (x + xline + ((y + yline) * 64)) as usize;
                            if pos < 2048
                            {
                                if self.gfx[pos] ==1
                                {
                                    self.v[0xF] = 1;
                                }
                                self.gfx[pos] ^= 1;
                            }
                        }
                    }
                }
            }

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

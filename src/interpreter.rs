// this should be read before reading the code
// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
// https://en.wikipedia.org/wiki/CHIP-8
use rand::Rng;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Emulator {
    memory: [u8; 4096],
    reg: [u8; 16],
    stack: [u16; 16],
    pc: u16,
    sp: u8,
    i: u16,
    delay_t: u8,
    sound_t: u8,
}

impl Default for Emulator {
    fn default() -> Self {
        Self {
            memory: [0; 4096],
            reg: [0; 16],
            stack: [0; 16],
            pc: 0,
            sp: 0,
            i: 0,
            delay_t: 0,
            sound_t: 0,
        }
    }
}

impl Emulator {
    fn push(&mut self, n: u16) {
        self.sp += 1;
        self.stack[self.sp as usize] = n;
    }
    fn pop(&mut self) -> u16 {
        let n = self.stack[self.sp as usize];
        self.sp -= 1;
        n
    }
    fn jmp(&mut self, location: u16) {
        self.pc = location;
    }

    fn next(&mut self) {
        self.pc += 1;
    }

    fn set_flag(&mut self, cond: bool) {
        self.reg[0xF] = if cond {
            1
        } else {
            0
        };
    }
}


pub fn run(program: &[u16]) {
    let mut em = Emulator::default();

    while em.pc < program.len() as u16 {
        let instruction = program[em.pc as usize];
        execute(instruction, &mut em);
        em.next();
    }
}

fn execute(instruction: u16, em: &mut Emulator) {
    match instruction {
        0x00E0 => unimplemented!(), // clear display
        0x00EE => { // return subroutine
            let location = em.pop();
            em.jmp(location);
        }
        0x1000..=0x1FFF => { // jmp
            let location = instruction & 0x0FFF;
            em.jmp(location - 1);
        }
        0x2000..=0x2FFF => { // 0nnn, call nnn
            let location = instruction & 0x0FFF;
            em.push(em.pc);
            em.jmp(location - 1);
        }
        0x3000..=0x3FFF => { // SE - Skip equal | 0xkk, skip instruction if Vx == kk
            let (x, kk) = extract_0xkk(instruction);
            if em.reg[x] == kk {
                em.next();
            }
        }
        0x4000..=0x4FFF => { // SNE - Skip not equal | 0xkk, skip instruction if Vx != kk
            let (x, kk) = extract_0xkk(instruction);
            if em.reg[x] != kk {
                em.next();
            }
        }
        0x5000..=0x5FF0 => { // SE - Skip equal | 0xy0, skip if Vx == Vy
            let (x, y) = extract_0xy0(instruction);
            if em.reg[x] == em.reg[y] {
                em.next();
            }
        }
        0x6000..=0x6FFF => { // LD - Load | 0xkk, load kk into Vx
            let (x, kk) = extract_0xkk(instruction);
            em.reg[x] = kk;
        }
        0x7000..=0x7FFF => { // ADD - Add | 0xkk, add kk to Vx
            let (x, kk) = extract_0xkk(instruction);
            em.reg[x] = em.reg[x].wrapping_add(kk);
        }
        0x8000..=0x8FF0 => { // 0xyz, do z on Vy, Vx
            let (x, y, z) = extract_0xyz(instruction);
            match z {
                0 => {  // LD - Load | Stores Vy in Vx
                    em.reg[x] = em.reg[y];
                }
                1 => { // OR | Stores a bitwise OR of Vx and Vy in Vx
                    em.reg[x] |= em.reg[y];
                }
                2 => { // AND | Stores a bitwise AND of Vx and Vy in Vx
                    em.reg[x] &= em.reg[y];
                }
                3 => { // XOR | Stores a bitwise XOR of Vx and Vy in Vx
                    em.reg[x] ^= em.reg[y];
                }
                4 => { // ADD | Adds Vx to Vy. If it overflows, Vf = 1 else Vf = 0
                    let (val, f) = em.reg[x].overflowing_add(em.reg[y]);
                    em.reg[x] = val;
                    em.set_flag(f);
                }
                5 => { // SUB | If Vx > Vy, Vf = 1, else Vf = 0, then subtract Vy from Vx and store it into Vx
                    em.set_flag(em.reg[x] > em.reg[y]);
                    em.reg[x] = em.reg[x].wrapping_sub(em.reg[y])
                }
                6 => { // SHR - shift right | shift Vx right by one bit, if the rightmost bit is 1 set flag
                    em.set_flag((em.reg[x] & 0x0001) > 0);
                    em.reg[x] >>= 1;
                }
                7 => { // SUBN - Sub not borrow | subtract Vx from Vy and store it into Vx
                    em.set_flag(em.reg[y] > em.reg[x]);
                    em.reg[x] = em.reg[y].wrapping_sub(em.reg[x]);
                }
                0xE => { // SHL - Shift left | shift Vx left, if the leftmost bit is 1 set flag
                    em.set_flag((em.reg[x] & 0x80) > 0);
                    em.reg[x] <<= 1;
                }
                _ => unreachable!("invalid instruction")
            }
        }
        0x9000..=0x9FFF => { // SNE - skip not equal | skip next if Vx != Vy
            let (x, y) = extract_0xy0(instruction);
            if em.reg[x] != em.reg[y] {
                em.next();
            }
        }
        0xA000..=0xAFFF => { // LDI - load i | load nnn into i
            let nnn = extract_0nnn(instruction);
            em.i = nnn;
        }
        0xB000..=0xBFFF => { // JMP | jump to nnn + V0
            let nnn = extract_0nnn(instruction);
            em.jmp(em.reg[0] as u16 + nnn - 1);
        }
        0xC000..=0xCFFF => { // RAN | set Vx to a random byte AND kk
            let (x, kk) = extract_0xii(instruction);
            let ran = rand::thread_rng().gen_range(0..=255);
            em.reg[x] = ran & kk;
        }
        0xD000..=0xDFFF => { // 0xyn display n-byte sprite starting at location I at (Vx, Vy), set Vf if there's a collision
            // read n bytes from memory at loc I and display it at coords (Vx, Vy)
            // XOR them onto the screen, Vf = 1 if a pixel was erased
            // wrap sprites around
            unimplemented!()
        }
        0xE000..=0xEFFF => { // skip/not skip instruction if key Vx is pressed
            let (x, sub_inst) = extract_0xii(instruction);
            fn key_pressed(key: u8) -> bool { unimplemented!("{}", key) }

            match sub_inst {
                0x9E => { // skip
                    if key_pressed(em.reg[x]) {
                        em.next();
                    }
                }
                0xA1 => { // not skip
                    if !key_pressed(em.reg[x]) {
                        em.next();
                    }
                }
                _ => unreachable!("invalid instruction")
            }
        }
        0xF000..=0xFFFF => { // misc
            let (x, sub_inst) = extract_0xii(instruction);
            #[allow(unreachable_code)]
            match sub_inst {
                0x0F => { // set Vx = delay timer value
                    em.reg[x] = em.delay_t;
                }
                0x0A => { // wait for key and store it in Vx
                    let _key = unimplemented!();
                    em.reg[x] = _key;
                }
                0x15 => { // set delay timer = Vx
                    em.delay_t = em.reg[x];
                }
                0x18 => { // set sound timer = Vx
                    em.sound_t = em.reg[x];
                }
                0x1E => { // Add Vx to I and store it in I
                    em.i += em.reg[x] as u16;
                }
                0x29 => { // set I to location for the sprite with value Vx
                    unimplemented!()
                }
                0x33 => { // store a decimal representation of Vx at I, I+1, I+2;
                    // convert Vx to decimal and store the digits, 100 -> I
                    let n = em.reg[x] as f32;
                    let hundreds = (n / 100.0).floor();
                    let n = n % 100.0;
                    let tens = (n / 10.0).floor();
                    let n = n % 10.0;

                    em.memory[em.i as usize] = hundreds as u8;
                    em.memory[(em.i + 1) as usize] = tens as u8;
                    em.memory[(em.i + 2) as usize] = n as u8;
                }
                0x55 => { // store registers V0 to Vx in memory I
                    for i in 0..x as u16 {
                        em.memory[(em.i + i) as usize] = em.reg[i as usize];
                    }
                }
                0x65 => { // load V0 to Vx from memory at I
                    for i in 0..x as u16 {
                        em.reg[i as usize] = em.memory[(em.i + i) as usize]
                    }
                }
                _ => unreachable!("invalid instruction")
            }
        }
        // nop for testing
        #[cfg(test)]
        0x0000 => {}
        _ => unreachable!("invalid instruction")
    }
}


fn extract_0xkk(value: u16) -> (usize, u8) {
    let x = (value & 0x0F00) >> 8;
    let kk = (value & 0x00FF) as u8;
    (x as usize, kk)
}

fn extract_0xy0(value: u16) -> (usize, usize) {
    let x = (value & 0x0F00) >> 8;
    let y = (value & 0x00F0) >> 4;
    (x as usize, y as usize)
}

fn extract_0xyz(value: u16) -> (usize, usize, u16) {
    let x = (value & 0x0F00) >> 8;
    let y = (value & 0x00F0) >> 4;
    let z = value & 0x000F;
    (x as usize, y as usize, z)
}

fn extract_0xii(value: u16) -> (usize, u8) {
    let x = (value & 0x0F00) >> 8;
    let ii = (value & 0x00FF) as u8;
    (x as usize, ii)
}

fn extract_0nnn(value: u16) -> u16 {
    value & 0xFFF
}

#[cfg(test)]
mod test {
    use crate::interpreter::{Emulator, execute};
    use std::sync::atomic::Ordering::AcqRel;

    #[test]
    fn instructions_0_1_2() {
        let mut em = Emulator::default();
        em.pc = 10;
        execute(0x20FF, &mut em); // call 0FF
        assert_eq!(em.pc + 1, 0x0FF);
        assert_eq!(em.sp, 1);
        em.pc = 0x0FF;
        execute(0x00EE, &mut em); // return
        assert_eq!(em.sp, 0);
        assert_eq!(em.pc, 10);
        execute(0x10AB, &mut em); // jmp to 0AB
        assert_eq!(em.pc + 1, 0x0AB);
    }

    #[test]
    fn instructions_3_4_5() {
        let mut em = Emulator::default();
        em.reg[0] = 0xF2;
        execute(0x30F2, &mut em);
        assert_eq!(em.pc, 1);
        execute(0x40F3, &mut em);
        assert_eq!(em.pc, 2);
        execute(0x40F2, &mut em);
        assert_eq!(em.pc, 2);
        em.reg[1] = 0xF2;
        execute(0x5010, &mut em);
        assert_eq!(em.pc, 3);
    }

    #[test]
    fn instructions_6_7() {
        let mut em = Emulator::default();
        execute(0x60AB, &mut em);
        assert_eq!(em.reg[0], 0xAB);
        execute(0x69FF, &mut em);
        assert_eq!(em.reg[9], 0xFF);
        execute(0x7001, &mut em);
        assert_eq!(em.reg[0], 0xAC);
    }

    #[test]
    fn instruction_fx33_conversion() {
        let mut em = Emulator::default();
        em.reg[0] = 255;
        em.reg[1] = 99;
        em.reg[2] = 1;
        let instruction1 = 0xF033;
        let instruction2 = 0xF133;
        let instruction3 = 0xF233;
        execute(instruction1, &mut em);
        em.i = 3;
        execute(instruction2, &mut em);
        em.i = 6;
        execute(instruction3, &mut em);

        assert_eq!(&em.memory[0..9], &[2, 5, 5, 0, 9, 9, 0, 0, 1]);
    }
}

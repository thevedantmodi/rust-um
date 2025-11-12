use crate::memory::Memory;
use std::fs::File;
use std::io::{self, Read};

pub struct UM {
    pub registers: [u32; 8],
    pub pc: usize,
    pub memory: Memory,
}
pub type UmWord = u32;
type UmInstruction = u32;
#[derive(PartialEq, Eq, Debug)]
pub enum UmOperations {
    /* Will work like a C enum with indexing if you cast with `as` */
    CMOV = 0,
    SLOAD = 1,
    SSTORE = 2,
    ADD = 3,
    MUL = 4,
    DIV = 5,
    NAND = 6,
    HALT = 7,
    MAP = 8,
    UNMAP = 9,
    OUT = 10,
    IN = 11,
    LOADP = 12,
    LV = 13,
}
pub type UmOp = UmOperations;

impl UM {
    pub fn new() -> Self {
        Self {
            registers: [0 as UmInstruction; 8],
            pc: 0,
            memory: Memory::new(),
        }
    }

    pub fn init_program(&mut self, path: &str) {
        let mut file = File::open(path).expect("Could not read program. Exiting...");

        let mut bytes = Vec::new();

        file.read_to_end(&mut bytes)
            .expect("Could not read program. Exiting...");

        let mut instructions: Vec<UmInstruction> = Vec::new();
        for i in (0..bytes.len()).step_by(4) {
            let word: UmWord =
                UmInstruction::from_be_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
            instructions.push(word);
        }

        self.memory.segments[0] = Some(instructions);
    }

    pub fn run(&mut self) {
        loop {
            let program = self.memory.segments[0].as_ref().unwrap();
            if self.pc >= program.len() {
                break;
            }

            /* fetch */
            let instr = program[self.pc];
            /* increment pc upon fetch */
            self.pc += 1;

            /* decode */
            let current_opcode = match instr >> 28 {
                0 => UmOperations::CMOV,
                1 => UmOperations::SLOAD,
                2 => UmOperations::SSTORE,
                3 => UmOperations::ADD,
                4 => UmOperations::MUL,
                5 => UmOperations::DIV,
                6 => UmOperations::NAND,
                7 => UmOperations::HALT,
                8 => UmOperations::MAP,
                9 => UmOperations::UNMAP,
                10 => UmOperations::OUT,
                11 => UmOperations::IN,
                12 => UmOperations::LOADP,
                13 => UmOperations::LV,
                _ => panic!("Invalid opcode in fetch: {}", instr >> 28),
            };
            match current_opcode {
                UmOperations::HALT => break,
                UmOperations::LV => {
                    let (a, val) = (((instr >> 25) & 0x7) as usize, instr & 0x1FFFFFF);
                    self.registers[a] = val;
                }
                _ => {
                    let (a, b, c) = (
                        ((instr >> 6) & 0x7) as usize,
                        ((instr >> 3) & 0x7) as usize,
                        (instr & 0x7) as usize,
                    );
                    self.execute(current_opcode, a, b, c);
                }
            }
        }
    }

    #[inline(always)]
    fn execute(&mut self, op: UmOp, a: usize, b: usize, c: usize) {
        match op {
            UmOperations::CMOV => {
                if self.registers[c] != 0 {
                    self.registers[a] = self.registers[b];
                }
            }
            UmOperations::SLOAD => {
                let seg = self.registers[b] as usize;
                let offset = self.registers[c] as usize;
                self.registers[a] = match &(self.memory.segments[seg]) {
                    Some(segment) => segment[offset],
                    None => panic!("No segment at {}", seg),
                }
            }
            UmOperations::SSTORE => {
                let seg = self.registers[a] as usize;
                let offset = self.registers[b] as usize;
                match &mut (self.memory.segments[seg]) {
                    Some(segment) => segment[offset] = self.registers[c],
                    None => panic!("No segment at {}", seg),
                }
            }
            UmOperations::ADD => {
                self.registers[a] = self.registers[b].wrapping_add(self.registers[c])
            }
            UmOperations::MUL => {
                self.registers[a] = self.registers[b].wrapping_mul(self.registers[c])
            }
            UmOperations::DIV => self.registers[a] = self.registers[b] / self.registers[c],
            UmOperations::NAND => self.registers[a] = !(self.registers[b] & self.registers[c]),
            UmOperations::MAP => {
                let size = self.registers[c] as usize;
                self.registers[b] = self.memory.map_segment(size) as u32;
            }
            UmOperations::UNMAP => self.memory.unmap_segment(self.registers[c] as usize),
            UmOperations::OUT => {
                print!("{}", (self.registers[c] & 0xFF) as u8 as char);
            }
            UmOperations::IN => {
                let mut buf = [0u8; 1];
                match io::stdin().read_exact(&mut buf) {
                    Ok(_) => self.registers[c] = buf[0] as u32,
                    Err(_) => self.registers[c] = 0xFFFFFFFFu32,
                }
            }
            UmOperations::LOADP => {
                if self.registers[b] != 0 {
                    let seg = self.registers[b] as usize;
                    let duplicate = match &(self.memory.segments[seg]) {
                        Some(segment) => segment.clone(),
                        None => panic!("No segment at {}", seg),
                    };
                    self.memory.segments[0] = Some(duplicate);
                }
                self.pc = self.registers[c] as usize;
            }
            _ => unreachable!(),
        }
    }
}

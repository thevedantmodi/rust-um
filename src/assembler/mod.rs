use crate::um::{UmOp, UmOperations, UmWord, UM};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

pub struct UMAssembler {}

impl UMAssembler {
    fn build_three_reg_instruction(opcode: u32, reg_a: u32, reg_b: u32, reg_c: u32) -> UmWord {
        let mut instruction = 0 as UmWord;
        instruction |= opcode << 28;
        instruction |= reg_a << 6;
        instruction |= reg_b << 3;
        instruction |= reg_c << 0; /* no shift */

        instruction
    }

    fn build_load_value_instruction(opcode: u32, reg_a: u32, value: u32) -> UmWord {
        let mut instruction = 0 as UmWord;
        instruction |= opcode << 28;
        instruction |= reg_a << 25;
        instruction |= value & 0x001FFFFFFu32; /* only capture the lower order 25 bits */

        instruction
    }

    fn parse_instruction(line: &str) -> Option<u32> {
        let line = line.trim().replace(" ", "");
        if let Some((left, right)) = line.split_once(":=") {
            if left.starts_with("m[") {
                let (segment_id, offset) = UMAssembler::parse_memory(left)?;
                let reg = UMAssembler::parse_reg(right)?;
                return Some(UMAssembler::build_three_reg_instruction(
                    UmOp::SSTORE as u32,
                    segment_id,
                    offset,
                    reg,
                ));
            }

            // left *must* be a register
            let lreg =
                UMAssembler::parse_reg(left).expect("Invalid left register in binop expression");

            // binop
            for (opcode_expr, opcode) in [
                ("+", UmOp::ADD),
                ("*", UmOp::MUL),
                ("/", UmOp::DIV),
                ("nand", UmOp::NAND),
            ] {
                if let Some((bin1, bin2)) = right.split_once(opcode_expr) {
                    let rb = UMAssembler::parse_reg(bin1)?;
                    let rc = UMAssembler::parse_reg(bin2)?;
                    return Some(UMAssembler::build_three_reg_instruction(
                        opcode as u32,
                        lreg,
                        rb,
                        rc,
                    ));
                }
            }

            // segmented load
            if let Some((segment_id, offset)) = UMAssembler::parse_memory(right) {
                return Some(UMAssembler::build_three_reg_instruction(
                    UmOp::SLOAD as u32,
                    lreg,
                    segment_id,
                    offset,
                ));
            }

            // cmov
            if let Some((src, test)) = UMAssembler::parse_condition(right) {
                return Some(UMAssembler::build_three_reg_instruction(
                    UmOp::CMOV as u32,
                    lreg,
                    src,
                    test,
                ));
            }

            // map
            if let Some(size) = UMAssembler::parse_size(right) {
                return Some(UMAssembler::build_three_reg_instruction(
                    UmOp::MAP as u32,
                    0,
                    lreg,
                    size,
                ));
            }
            // load value
            if let Some(value) = UMAssembler::parse_value(right) {
                return Some(UMAssembler::build_load_value_instruction(
                    UmOp::LV as u32,
                    lreg,
                    value,
                ));
            }
        }
        // unop
        for (unop_expr, opcode) in [
            ("unmap", UmOperations::UNMAP),
            ("out", UmOperations::OUT),
            ("in", UmOperations::IN),
        ] {
            if let Some(reg) = line.strip_prefix(unop_expr) {
                let rc = UMAssembler::parse_reg(reg)?;
                return Some(UMAssembler::build_three_reg_instruction(
                    opcode as u32,
                    0,
                    0,
                    rc,
                ));
            }
        }

        if let Some(inner_expr) = line.strip_prefix("goto") {
            if let Some((segment_id, offset)) = UMAssembler::parse_memory(inner_expr) {
                return Some(UMAssembler::build_three_reg_instruction(
                    UmOp::LOADP as u32,
                    0,
                    segment_id,
                    offset,
                ));
            }
        }

        if let Some(_) = line.strip_prefix("halt") {
            return Some(UMAssembler::build_three_reg_instruction(
                UmOp::HALT as u32,
                0,
                0,
                0,
            ));
        }

        None
    }

    fn parse_value(line: &str) -> Option<u32> {
        if let Some(hex) = line.strip_prefix("0x") {
            u32::from_str_radix(hex, 16).ok()
        } else if let Some(bin) = line.strip_prefix("0b") {
            u32::from_str_radix(bin, 2).ok()
        } else {
            line.parse::<u32>().ok()
        }
    }
    fn parse_size(line: &str) -> Option<u32> {
        if let Some(inner_expr) = line.strip_prefix("map") {
            if let Some(reg) = UMAssembler::parse_reg(inner_expr) {
                return Some(reg);
            }
        }

        None
    }
    fn parse_condition(line: &str) -> Option<(u32, u32)> {
        if let Some((r1, r2)) = line.split_once("if") {
            return Some((UMAssembler::parse_reg(r1)?, UMAssembler::parse_reg(r2)?));
        }

        None
    }

    /* returns the segment id and the offset iff succeeded */
    fn parse_memory(line: &str) -> Option<(u32, u32)> {
        if line.starts_with("m[") {
            let inner_expr = &line[2..line.len() - 1];
            if let Some((r1, r2)) = inner_expr.split_once("][") {
                return Some((UMAssembler::parse_reg(r1)?, UMAssembler::parse_reg(r2)?));
            }
        }

        None
    }

    /* expects no whitespaces */
    fn parse_reg(line: &str) -> Option<u32> {
        if line.chars().any(|c| c.is_whitespace()) {
            return None;
        }

        let digits = line.strip_prefix('r')?;

        if digits.is_empty() {
            return None;
        }

        digits.parse::<u32>().ok()
    }

    pub fn read_asm_code(&mut self, path: &str) -> io::Result<Vec<u32>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut instructions = Vec::new();

        for (i, line) in reader.lines().enumerate() {
            match UMAssembler::parse_instruction(&line?) {
                Some(instr) => instructions.push(instr),
                None => panic!("could not process instruction {}", i),
            }
        }

        Ok(instructions)
    }

    pub fn write_mach_code(&mut self, program: &Vec<u32>, opath: &str) -> io::Result<()> {
        let mut file = File::create(opath)?;
        for (i, instr) in program.iter().enumerate() {
            let bytes = instr.to_be_bytes(); // big-endian byte array [b0, b1, b2, b3]
            file.write_all(&bytes)?;
        }

        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use crate::assembler::UMAssembler;
    use crate::um::UmOperations;

    #[test]
    fn test_parse_reg() {
        for i in 1..100 {
            assert_eq!(UMAssembler::parse_reg(&format!("r{}", i)), Some(i))
        }
    }
    #[test]
    fn test_parse_instruction() {
        assert_eq!(
            UMAssembler::parse_instruction("r1 := r2 + r3"),
            Some(UMAssembler::build_three_reg_instruction(
                UmOperations::ADD as u32,
                1,
                2,
                3
            ))
        );
    }
    #[test]
    fn test_parse_instruction2() {
        assert_eq!(
            UMAssembler::parse_instruction("r99 := r2 + r3"),
            Some(UMAssembler::build_three_reg_instruction(
                UmOperations::ADD as u32,
                99,
                2,
                3
            ))
        );
    }
    #[test]
    fn test_parse_instruction3() {
        assert_eq!(
            UMAssembler::parse_instruction("r99 := r2 nand r3"),
            Some(UMAssembler::build_three_reg_instruction(
                UmOperations::NAND as u32,
                99,
                2,
                3
            ))
        );
    }
    #[test]
    fn test_parse_instruction4() {
        assert_eq!(
            UMAssembler::parse_instruction("r99 := r2 * r3"),
            Some(UMAssembler::build_three_reg_instruction(
                UmOperations::MUL as u32,
                99,
                2,
                3
            ))
        );
    }
    #[test]
    fn test_parse_instruction5() {
        assert_eq!(
            UMAssembler::parse_instruction("r99 := r2 / r3"),
            Some(UMAssembler::build_three_reg_instruction(
                UmOperations::DIV as u32,
                99,
                2,
                3
            ))
        );
    }
    #[test]
    fn test_parse_instruction6() {
        assert_eq!(
            UMAssembler::parse_instruction("r99 := m[r2][r3]"),
            Some(UMAssembler::build_three_reg_instruction(
                UmOperations::SLOAD as u32,
                99,
                2,
                3
            ))
        );
    }
    #[test]
    fn test_parse_instruction7() {
        assert_eq!(
            UMAssembler::parse_instruction("r1 := r2 if r3"),
            Some(UMAssembler::build_three_reg_instruction(
                UmOperations::CMOV as u32,
                1,
                2,
                3
            ))
        );
    }
    #[test]
    fn test_parse_instruction8() {
        assert_eq!(
            UMAssembler::parse_instruction("r1 := map r6"),
            Some(UMAssembler::build_three_reg_instruction(
                UmOperations::MAP as u32,
                0,
                1,
                6
            ))
        );
    }
    #[test]
    fn test_parse_instruction9() {
        assert_eq!(
            UMAssembler::parse_instruction("r1 := 55"),
            Some(UMAssembler::build_load_value_instruction(
                UmOperations::LV as u32,
                1,
                55
            ))
        );
    }
    #[test]
    fn test_parse_instruction10() {
        assert_eq!(
            UMAssembler::parse_instruction("r1 := 0x55"),
            Some(UMAssembler::build_load_value_instruction(
                UmOperations::LV as u32,
                1,
                0x55
            ))
        );
    }
    #[test]
    fn test_parse_instruction11() {
        assert_eq!(
            UMAssembler::parse_instruction("r1 := 0b11"),
            Some(UMAssembler::build_load_value_instruction(
                UmOperations::LV as u32,
                1,
                0b11
            ))
        );
    }
    #[test]
    fn test_parse_instruction12() {
        assert_eq!(
            UMAssembler::parse_instruction("unmap r1"),
            Some(UMAssembler::build_three_reg_instruction(
                UmOperations::UNMAP as u32,
                0,
                0,
                1
            ))
        );
    }
    #[test]
    fn test_parse_instruction13() {
        assert_eq!(
            UMAssembler::parse_instruction("out r1"),
            Some(UMAssembler::build_three_reg_instruction(
                UmOperations::OUT as u32,
                0,
                0,
                1
            ))
        );
    }
    #[test]
    fn test_parse_instruction14() {
        assert_eq!(
            UMAssembler::parse_instruction("out r1"),
            Some(UMAssembler::build_three_reg_instruction(
                UmOperations::OUT as u32,
                0,
                0,
                1
            ))
        );
    }
    #[test]
    fn test_parse_instruction15() {
        assert_eq!(
            UMAssembler::parse_instruction("in r1"),
            Some(UMAssembler::build_three_reg_instruction(
                UmOperations::IN as u32,
                0,
                0,
                1
            ))
        );
    }

    #[test]
    fn test_parse_instruction_sstore() {
        assert_eq!(
            UMAssembler::parse_instruction("m[r1][r2] := r3"),
            Some(UMAssembler::build_three_reg_instruction(
                UmOperations::SSTORE as u32,
                1,
                2,
                3
            ))
        );
    }

    #[test]
    fn test_parse_instruction_lp() {
        assert_eq!(
            UMAssembler::parse_instruction("goto m[r2][r5]"),
            Some(UMAssembler::build_three_reg_instruction(
                UmOperations::LOADP as u32,
                0,
                2,
                5
            ))
        );
    }
    #[test]
    fn test_parse_instruction_halt() {
        assert_eq!(
            UMAssembler::parse_instruction("halt"),
            Some(UMAssembler::build_three_reg_instruction(
                UmOperations::HALT as u32,
                0,
                0,
                0
            ))
        );
    }
}

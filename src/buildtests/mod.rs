use crate::um::{UmOp, UmWord};
#[derive(Clone, Copy)]
pub struct TestBuilder {

}

impl TestBuilder {
    fn build_three_reg_instruction(
        self,
        opcode: u32,
        reg_a: u32,
        reg_b: u32,
        reg_c: u32,
    ) -> UmWord {
        let mut instruction = 0 as UmWord;
        instruction |= opcode << 28;
        instruction |= reg_a << 6;
        instruction |= reg_b << 3;
        instruction |= reg_c << 0; /* no shift */

        instruction
    }

    fn build_load_value_instruction(self, opcode: u32, reg_a: u32, value: u32) -> UmWord {
        let mut instruction = 0 as UmWord;
        instruction |= opcode << 28;
        instruction |= reg_a << 25;
        instruction |= value & 0x001FFFFFFu32; /* only capture the lower order 25 bits */

        instruction
    }

    pub fn writetests(&mut self) {
        assert_eq!(self.build_three_reg_instruction(0, 0, 0, 0), 0x00000000u32);
        assert_eq!(self.build_three_reg_instruction(1, 0, 0, 0), 0x10000000u32);
        assert_eq!(
            self.build_three_reg_instruction(1, 0, 7, 1),
            0b00010000000000000000000000111001
        );
        assert_eq!(
            self.build_three_reg_instruction(10, 0, 7, 1),
            0b10100000000000000000000000111001
        );
        assert_eq!(
            self.build_three_reg_instruction(10, 1, 7, 1),
            0b10100000000000000000000001111001
        );
        assert_eq!(self.build_load_value_instruction(13, 2, 5), 0xD4000005u32);
        assert_eq!(self.build_load_value_instruction(13, 3, 5), 0xD6000005u32);
        assert_eq!(
            self.build_load_value_instruction(13, 3, 0xFFFFFFFFu32),
            0xD7FFFFFFu32
        ); /* don't turn off my bits! */

        
    }
}

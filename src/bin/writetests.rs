use um::assembler;
use std::io;
// mod tests;

fn main() -> io::Result<()> {
    let mut assembler_module = assembler::UMAssembler {};

    let program = assembler_module.read_asm_code("add_two_numbers.ums")?;

    assembler_module.write_mach_code(&program, "add_two_numbers.um")?;
    
    Ok(())
}

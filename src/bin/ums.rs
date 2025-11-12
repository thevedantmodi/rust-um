use std::{
    env,
    io::{self, Error, ErrorKind},
};
use um::assembler;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Not enough arguments provided.",
        ));
    }

    for i in 1..args.len() {
        let mut assembler_module = assembler::UMAssembler {};
        match args[i].split_once(".") {
            // Some ((l, r)) => eprintln!("{} {}", l, r),
            // None => eprint!("yo")
            Some((base, extension)) if extension == "ums" => {
                println!("Writing {}.um", base);
                let program = assembler_module.read_asm_code(&args[i])?;
                let opath = String::from(base) + ".um";
                assembler_module.write_mach_code(&program, &opath)?;
            }
            _ => eprintln!("Warning: Skipping non ums file"),
        }
    }

    Ok(())
}

mod memory;
mod um;

use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <program.um>", args[0]);
        process::exit(1);
    }

    let mut machine = um::UM::new();
    machine.init_program(&args[1]);
    machine.run();
}


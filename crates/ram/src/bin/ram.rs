use std::process::ExitCode;

use miette::*;

fn main() -> Result<ExitCode> {
    ram::main(std::env::args_os())
}

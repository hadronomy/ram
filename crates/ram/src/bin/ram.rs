use std::process::ExitCode;

use miette::*;

#[tokio::main]
async fn main() -> Result<ExitCode> {
    ramlang::main(std::env::args_os()).await
}

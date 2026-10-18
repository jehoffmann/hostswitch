// SPDX-License-Identifier: Apache-2.0

mod args;
mod commands;
mod render;

use std::process::ExitCode;

use clap::Parser;

use args::Cli;

#[tokio::main]
async fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            let code = if e.use_stderr() { 64 } else { 0 };
            let _ = e.print();
            return ExitCode::from(code);
        }
    };
    ExitCode::from(commands::run(cli).await)
}

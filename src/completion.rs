use clap::CommandFactory;
use clap_complete::{generate as clap_generate, Shell};

use crate::cli::Cli;

pub fn generate(shell: Shell) {
    let mut cmd = Cli::command();

    clap_generate(shell, &mut cmd, "hyprdeck", &mut std::io::stdout());
}

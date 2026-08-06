pub mod volume;

use crate::cli::AudioCommands;

pub fn execute(commands: AudioCommands) -> Result<(), Box<dyn std::error::Error>> {
    match commands {
        AudioCommands::Volume { commands } => {
            volume::execute_volume(commands)?;
        }
    }

    Ok(())
}

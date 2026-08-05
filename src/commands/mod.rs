pub mod modules;

use crate::cli::Commands;

pub fn execute(command: Commands) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::Touchpad { commands } => {
            let touchpad_device_id = modules::touchpad::get_touchpad_device_id()?;
            modules::touchpad::execute_touchpad(touchpad_device_id, commands)?;
            Ok(())
        }

        Commands::Volume { commands } => {
            modules::volume::execute_volume(commands)?;
            Ok(())
        }

        _ => Ok(()),
    }
}

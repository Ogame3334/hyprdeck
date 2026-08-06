pub mod modules;

use crate::cli::Commands;

pub fn execute(command: Commands) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::Touchpad { commands } => {
            let touchpad_device_id = modules::touchpad::get_touchpad_device_id()?;
            modules::touchpad::execute_touchpad(touchpad_device_id, commands)?;
            Ok(())
        }

        Commands::Audio { commands } => {
            modules::audio::execute(commands)?;
            Ok(())
        }

        Commands::Wifi { commands } => {
            modules::wifi::execute(commands)?;
            Ok(())
        }

        _ => Ok(()),
    }
}

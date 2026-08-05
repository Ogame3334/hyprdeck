pub mod input;

use crate::cli::Commands;

pub fn execute(command: Commands) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::Touchpad { commands } => {
            let touchpad_device_id = input::touchpad::get_touchpad_device_id()?;
            input::touchpad::execute_touchpad(touchpad_device_id, commands)?;
            Ok(())
        }
        _ => Ok(()),
    }
}

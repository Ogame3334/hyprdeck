<h1>hyprdeck <a href="https://github.com/Ogame3334/hyprdeck/blob/main/LICENCE"><img src="https://img.shields.io/badge/license-MIT-4aaa4a"></a></h1>

English | [日本語](https://github.com/Ogame3334/hyprdeck/blob/main/docs/README_jp.md)

### Overview

A CLI tool for operating various Hyprland settings from the command line.
A self-made personal environment tool; no guarantee that it works in other environments.

## Features

- **Touchpad control** - Touchpad ON/OFF/toggle/status check (via `hyprctl`)
- **Volume control** - Get/set the volume of the default audio sink (via `wpctl`)
- **Screenshot capture** - Select a region, capture the desktop/window/monitor, save PNGs and copy them to the Wayland clipboard
- **Battery status** - Read battery, charging, power, health and AC state directly from Linux sysfs
- **System metrics** - Show CPU, temperature, memory, storage and network usage from Linux procfs/sysfs
- **Wi-Fi control** - Connect interactively, inspect devices and nearby networks, manage saved connections, and toggle/disconnect Wi-Fi through NetworkManager
- **Clipboard control** - Copy, paste, inspect MIME types, and clear Wayland clipboard contents
- **Shell completion** - Generates completion scripts for bash / zsh / fish / elvish / powershell

## Requirements

- Linux + Hyprland
- [hyprctl](https://github.com/hyprwm/Hyprland) (bundled with Hyprland)
- [wpctl](https://github.com/PipeWire/wireplumber) (bundled with WirePlumber)
- `grim` and `slurp` (required for screenshots)
- `wl-clipboard` (required only for `screenshot --clipboard`)
- `hyprpicker` (optional, required for `screenshot --freeze`)
- `nmcli` (required for Wi-Fi commands; provided by NetworkManager)

## Install

```sh
cargo install --git https://github.com/Ogame3334/hyprdeck
```

## Usage

### Touchpad

```sh
# ON / OFF / toggle
hyprdeck touchpad on
hyprdeck touchpad off
hyprdeck touchpad toggle

# Check status
hyprdeck touchpad status
hyprdeck touchpad status --verbose   # Also shows device name and state file path
```

The touchpad state is stored in `$XDG_STATE_HOME/hyprdeck/touchpad-disabled`.

### Audio

```sh
# Get the volume (same output as wpctl)
hyprdeck audio volume status

# Show as a percentage
hyprdeck audio volume status --percent

# Show as a value between 0.0 and 1.0
hyprdeck audio volume status --value

# Set the volume (percentage or 0.0-1.0)
hyprdeck audio volume set 50%
hyprdeck audio volume set 0.8

# Increase/decrease the default output by 5% (or a specified amount)
hyprdeck audio volume increase
hyprdeck audio volume decrease 10%

# Mute controls
hyprdeck audio volume mute
hyprdeck audio volume unmute
hyprdeck audio volume toggle-mute

# Inspect another PipeWire node and get machine-readable output
hyprdeck audio volume status --device alsa_output.pci-0000_00_1f.3.analog-stereo --json
```

### Shell completion

```sh
# bash
hyprdeck completion bash
# zsh
hyprdeck completion zsh
# fish
hyprdeck completion fish
```

### Screenshot

```sh
hyprdeck screenshot
hyprdeck screenshot --fast
hyprdeck screenshot --window
hyprdeck screenshot --monitor DP-1
hyprdeck screenshot --output ~/Pictures/share.png --clipboard
hyprdeck screenshot --no-save --clipboard
hyprdeck screenshot --freeze
```

### Battery

```sh
hyprdeck battery status
hyprdeck battery status --short
hyprdeck battery status --verbose
hyprdeck battery status --json
hyprdeck battery status --battery BAT0
```

### System metrics

```sh
hyprdeck metrics status
hyprdeck metrics status --cpu --memory
hyprdeck metrics status --json
hyprdeck metrics status --short
hyprdeck metrics status --storage --mount /
hyprdeck metrics watch --interval 2
```

### Wi-Fi

```sh
hyprdeck wifi connect
hyprdeck wifi status
hyprdeck wifi status --json
hyprdeck wifi scan --rescan
hyprdeck wifi scan --json
hyprdeck wifi saved
hyprdeck wifi disconnect
hyprdeck wifi disconnect wlan0
hyprdeck wifi on
hyprdeck wifi off
```

### Clipboard

Clipboard commands read and write standard input/output, so they work with text and binary data.

```sh
printf 'hello' | hyprdeck clipboard copy
hyprdeck clipboard paste --no-newline
hyprdeck clipboard show
hyprdeck clipboard types
hyprdeck clipboard clear
cat image.png | hyprdeck clipboard copy --mime-type image/png
hyprdeck clipboard paste --mime-type image/png > image.png
```

`clipboard show` displays text contents. If the clipboard only contains binary data, it prints the available MIME types instead of writing binary bytes to the terminal.

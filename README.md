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
- **Shell completion** - Generates completion scripts for bash / zsh / fish / elvish / powershell

## Requirements

- Linux + Hyprland
- [hyprctl](https://github.com/hyprwm/Hyprland) (bundled with Hyprland)
- [wpctl](https://github.com/PipeWire/wireplumber) (bundled with WirePlumber)
- `grim` and `slurp` (required for screenshots)
- `wl-clipboard` (required only for `screenshot --clipboard`)
- `hyprpicker` (optional, required for `screenshot --freeze`)

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

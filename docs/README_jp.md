<h1>hyprdeck <a href="https://github.com/Ogame3334/hyprdeck/blob/main/LICENCE"><img src="https://img.shields.io/badge/license-MIT-4aaa4a"></a><h1>

Hyprland の各種設定をコマンドラインから操作するための CLI ツール。
他の環境での動作は一切保証しない、オレオレ環境ツール。

## Features

- **タッチパッド制御** - タッチパッドの ON/OFF/トグル/ステータス確認 (`hyprctl` 経由)
- **音量制御** - デフォルトオーディオシンクの音量取得・設定 (`wpctl` 経由)
- **シェル補完** - bash / zsh / fish / elvish / powershell 用の補完スクリプト生成

## Requirements

- Linux + Hyprland
- [hyprctl](https://github.com/hyprwm/Hyprland) (Hyprland に同梱)
- [wpctl](https://github.com/PipeWire/wireplumber) (WirePlumber に同梱)

## Install

```sh
cargo install --git https://github.com/Ogame3334/hyprdeck
```

## Usage

### Touchpad

```sh
# ON / OFF / トグル
hyprdeck touchpad on
hyprdeck touchpad off
hyprdeck touchpad toggle

# ステータス確認
hyprdeck touchpad status
hyprdeck touchpad status --verbose   # デバイス名や状態ファイルのパスも表示
```

タッチパッドの状態は `$XDG_STATE_HOME/hyprdeck/touchpad-disabled` に保存されます。

### Audio

```sh
# 音量を取得 (wpctl と同じ出力)
hyprdeck audio volume status

# パーセント表示
hyprdeck audio volume status --percent

# 0.0〜1.0 の値で表示
hyprdeck audio volume status --value

# 音量を設定 (パーセントまたは 0.0〜1.0)
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

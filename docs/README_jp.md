<h1>hyprdeck <a href="https://github.com/Ogame3334/hyprdeck/blob/main/LICENCE"><img src="https://img.shields.io/badge/license-MIT-4aaa4a"></a></h1>

[English](https://github.com/Ogame3334/hyprdeck/blob/main/README.md) | 日本語

### Overview

Hyprland の各種設定をコマンドラインから操作するための CLI ツール。
他の環境での動作は一切保証しない、オレオレ環境ツール。

## Features

- **タッチパッド制御** - タッチパッドの ON/OFF/トグル/ステータス確認 (`hyprctl` 経由)
- **音量制御** - デフォルトオーディオシンクの音量取得・設定 (`wpctl` 経由)
- **スクリーンショット** - 範囲、デスクトップ、ウィンドウ、モニターの撮影、PNG 保存、Wayland クリップボードへのコピー
- **バッテリー情報** - Linux の sysfs からバッテリー、充電状態、電力、劣化度、AC 接続を取得
- **システムメトリクス** - Linux の procfs/sysfs から CPU、温度、メモリ、ストレージ、ネットワーク使用量を取得
- **Wi-Fi 制御** - 対話的な接続、デバイス・周辺ネットワーク確認、保存済み接続管理、Wi-Fi の有効化・切断 (`NetworkManager` 経由)
- **クリップボード制御** - Wayland クリップボードへのコピー、貼り付け、MIME type 確認、消去
- **シェル補完** - bash / zsh / fish / elvish / powershell 用の補完スクリプト生成

## Requirements

- Linux + Hyprland
- [hyprctl](https://github.com/hyprwm/Hyprland) (Hyprland に同梱)
- [wpctl](https://github.com/PipeWire/wireplumber) (WirePlumber に同梱)
- `grim` と `slurp` (スクリーンショットに必須)
- `wl-clipboard` (`screenshot --clipboard` に必須)
- `hyprpicker` (`screenshot --freeze` に必須)
- `nmcli` (Wi-Fi 機能に必須、NetworkManager に同梱)

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

# デフォルト出力を 5% 増減 (任意の値も指定可能)
hyprdeck audio volume increase
hyprdeck audio volume decrease 10%

# ミュート操作
hyprdeck audio volume mute
hyprdeck audio volume unmute
hyprdeck audio volume toggle-mute

# 別の PipeWire ノードを確認し、JSON で出力
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

### スクリーンショット

```sh
hyprdeck screenshot
hyprdeck screenshot --fast
hyprdeck screenshot --window
hyprdeck screenshot --monitor DP-1
hyprdeck screenshot --output ~/Pictures/share.png --clipboard
hyprdeck screenshot --no-save --clipboard
hyprdeck screenshot --freeze
```

### バッテリー

```sh
hyprdeck battery status
hyprdeck battery status --short
hyprdeck battery status --verbose
hyprdeck battery status --json
hyprdeck battery status --battery BAT0
```

### システムメトリクス

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

### クリップボード

標準入力・標準出力を使うため、テキストとバイナリの両方を扱えます。

```sh
printf 'hello' | hyprdeck clipboard copy
hyprdeck clipboard paste --no-newline
hyprdeck clipboard show
hyprdeck clipboard types
hyprdeck clipboard clear
cat image.png | hyprdeck clipboard copy --mime-type image/png
hyprdeck clipboard paste --mime-type image/png > image.png
```

`clipboard show` はテキストの内容を表示します。バイナリだけの場合は端末へバイナリを出力せず、利用可能な MIME type を表示します。

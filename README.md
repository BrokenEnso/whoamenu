# whoamenu

A small dmenu-like launcher written in **.NET** using **Avalonia UI**.

## Features

- Reads menu entries from `stdin`
- Input only mode when nothing in piped in
- Filters entries as you type
- Keyboard navigation with `Up` / `Down`
- `Enter` accepts selected item and prints to `stdout`
- `Esc` cancels (exit code `1`)
- Optional case-sensitive matching
- Frameless, always-on-top window

<img width="899" height="382" alt="whoamenu-v0 1 0-example config" src="https://github.com/user-attachments/assets/0070f890-b0c1-489f-b8a8-d13fd4e860f2" />


## Usage

Select a program
```bash
printf "firefox\nnotepad\ncalc\n" | whoamenu- -p "Program:"
```

Get input
```bash
whoamenu- -p "What is Sen's real name?"
```

### Flags

Configuration is loaded from `$XDG_CONFIG_HOME/whoamenu/config`.
If `XDG_CONFIG_HOME` is unset, the fallback path is `$HOME/.config/whoamenu/config`.
Command-line flags always override values from the configuration file.

- `-p` prompt text (default: `>`)
- `-case-sensitive` enable case-sensitive filtering
- `-font-size` set font size (default: `12`)
- `-fn` set font family name used throughout the app (default: platform default)
- `-m` choose monitor number (1-based, default: `1`)
- `-b` place the menu at the bottom of the selected monitor's working area
- `-t` place the menu at the top of the selected monitor's working area
- `-l` set number of visible lines and adjust window height (default: `10`)
- `-rc [radius]` set window corner radius; if omitted, no corner radius value is applied
- `-tr` set window transparency/opacity value (`0` to `1`)
- `-nb` set normal background color (`#RGB`, `#RRGGBB`, or color names)
- `-nf` set normal foreground color (`#RGB`, `#RRGGBB`, or color names)
- `-sb` set selected item background color (`#RGB`, `#RRGGBB`, or color names)
- `-sf` set selected item foreground color (`#RGB`, `#RRGGBB`, or color names)

## Build

```bash
dotnet build
```

For a self-contained executable, publish for your target runtime:

```bash
dotnet publish -c Release -r win-x64 --self-contained true
```

## Acknoledgements

- Example color scheme: https://github.com/catppuccin/dmenu/
- Example font: https://github.com/ryanoasis/nerd-fonts
- Inspiration: https://tools.suckless.org/dmenu/
- Contagious enthusiasm: https://github.com/BreadOnPenguins/

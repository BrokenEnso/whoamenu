# whoamenu

A small dmenu-like launcher written in **.NET** using **Avalonia UI**.

## Features

- Reads menu entries from `stdin`
- Filters entries as you type
- Keyboard navigation with `Up` / `Down`
- `Enter` accepts selected item and prints to `stdout`
- `Esc` cancels (exit code `1`)
- Optional case-sensitive matching
- Frameless, always-on-top window

## Usage

```bash
printf "firefox\nnotepad\ncalc\n" | dotnet run -- -p "run: "
```

### Flags

- `-p` prompt text (default: `>`)
- `-case-sensitive` enable case-sensitive filtering
- `-font-size` set font size (default: `12`)
- `-fn` set font family name used throughout the app (default: platform default)
- `-m` choose monitor number (1-based, default: `1`)
- `-b` place the menu at the bottom of the selected monitor's working area
- `-t` place the menu at the top of the selected monitor's working area
- `-l` set number of visible lines and adjust window height (default: `10`)
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

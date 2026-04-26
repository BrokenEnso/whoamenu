# whoamenu

*whoamenu* is a small desktop launcher built with Rust and `eframe`/`egui`. It works like dmenu.

It reads a list of options from standard input (for example, from a script) or runs on its own, then displays a simple always-on-top window where you can type to filter options in real time and select an item from the list. It can also be used to collect simple user input for scripts.

Configuration can be loaded from a config file and overridden with command-line flags.

## Features

- Reads menu entries from `stdin`
- Input-only mode when nothing is piped in
- Filters entries as you type
- Keyboard navigation with `Up` / `Down`
- `Enter` accepts selected item and prints to `stdout`
- `Esc` cancels (exit code `1`)
- Optional case-sensitive matching
- Frameless, always-on-top window

## Usage

Select a program:

```bash
printf "firefox\nnotepad\ncalc\n" | whoamenu -p "Program:"
```

Get input:

```bash
whoamenu -p "What is Sen's real name?"
```

### Flags

Configuration is loaded from `$XDG_CONFIG_HOME/whoamenu/config`.
If `XDG_CONFIG_HOME` is unset, the fallback path is `$HOME/.config/whoamenu/config`.
Command-line flags always override values from the configuration file.

- `-h` show help
- `-p` prompt text (default: `>`)
- `-clip` copy selected output to clipboard in addition to printing to `stdout`
- `-case-sensitive` enable case-sensitive filtering
- `-font-size` set font size (default: `12`)
- `-fn` set font family name
- `-m` choose monitor number (1-based)
- `-b` place the menu near the bottom (mutually exclusive with `-t`)
- `-t` place the menu near the top (mutually exclusive with `-b`)
- `-l` set number of visible lines (default: `10`)
- `-vs` set vertical space between list items in pixels (default: `0`)
- `-rc [radius]` set window corner radius
- `-tr` set window transparency/opacity value (`0` to `1`)
- `-nb` set normal background color (`#RGB`, `#RRGGBB`, or color names)
- `-nf` set normal foreground color (`#RGB`, `#RRGGBB`, or color names)
- `-sb` set selected item background color (`#RGB`, `#RRGGBB`, or color names)
- `-sf` set selected item foreground color (`#RGB`, `#RRGGBB`, or color names)

## Build

```bash
cargo build
```

Release build:

```bash
cargo build --release
```

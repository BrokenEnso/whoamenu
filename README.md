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
- `-m` choose monitor number (1-based, default: `1`)
- `-b` place the menu at the bottom of the selected monitor's working area

## Build

```bash
dotnet build
```

For a self-contained executable, publish for your target runtime:

```bash
dotnet publish -c Release -r win-x64 --self-contained true
```

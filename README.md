# whoamenu

A small dmenu-like launcher written in Go, now using **Wails** for the UI.

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
printf "firefox\nnotepad\ncalc\n" | go run . -p "run: "
```

### Flags

- `-p` prompt text (default: `>`)
- `-case-sensitive` enable case-sensitive filtering
- `-font-size` set font size (default: `12`)

## Build

```bash
go build -o whoamenu .
```

For a Windows executable:

```bash
go build -o whoamenu.exe .
```

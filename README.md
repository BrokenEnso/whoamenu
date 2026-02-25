# whoamenu

A small dmenu-like launcher written in Go, using **TK9.0** for the UI.

## Features

- Reads menu entries from `stdin`
- Filters entries as you type
- Keyboard navigation with `Up` / `Down`
- `Enter` accepts selected item and prints to `stdout`
- `Esc` cancels (exit code `1`)
- Optional case-sensitive matching

## Usage

```bash
printf "firefox\nnotepad\ncalc\n" | go run . -p "run: "
```

### Flags

- `-p` prompt text (default: `>`)
- `-case-sensitive` enable case-sensitive filtering
- `-font-size` set font size (default: `12`)

## Notes for Windows

The app is implemented with TK9.0 and is intended to work on Windows.
Build an executable with:

```bash
go build -o whoamenu.exe .
```

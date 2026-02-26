package main

import (
	"bufio"
	"embed"
	"flag"
	"fmt"
	"io"
	"os"
	"strings"

	"github.com/wailsapp/wails/v2"
	"github.com/wailsapp/wails/v2/pkg/options"
	"github.com/wailsapp/wails/v2/pkg/options/assetserver"
)

//go:embed frontend/dist
var assets embed.FS

func main() {
	prompt := flag.String("p", ">", "prompt text")
	caseSensitive := flag.Bool("case-sensitive", false, "use case-sensitive filtering")
	fontSize := flag.Int("font-size", 12, "font size")
	flag.Parse()

	items, err := readItems(os.Stdin)
	if err != nil {
		fmt.Fprintf(os.Stderr, "failed to read input: %v\n", err)
		os.Exit(1)
	}

	app := NewApp(items, *prompt, *caseSensitive, *fontSize)

	err = wails.Run(&options.App{
		Title:         "whoamenu",
		Width:         720,
		Height:        360,
		DisableResize: true,
		AlwaysOnTop:   true,
		Frameless:     true,
		AssetServer:   &assetserver.Options{Assets: assets},
		OnStartup:     app.startup,
		OnBeforeClose: app.beforeClose,
	})
	if err != nil {
		fmt.Fprintf(os.Stderr, "ui error: %v\n", err)
		os.Exit(1)
	}

	if app.accept {
		fmt.Println(app.result)
		return
	}

	os.Exit(1)
}

func readItems(r io.Reader) ([]string, error) {
	var items []string
	s := bufio.NewScanner(r)
	for s.Scan() {
		line := strings.TrimSpace(s.Text())
		if line == "" {
			continue
		}
		items = append(items, line)
	}
	if err := s.Err(); err != nil {
		return nil, err
	}
	return items, nil
}

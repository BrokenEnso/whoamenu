package main

import (
	"context"
	"strings"

	"github.com/wailsapp/wails/v2/pkg/runtime"
)

type appState struct {
	PromptText    string   `json:"promptText"`
	Input         string   `json:"input"`
	FilteredItems []string `json:"filteredItems"`
	SelectedIndex int      `json:"selectedIndex"`
	FontSize      int      `json:"fontSize"`
}

type App struct {
	ctx context.Context

	allItems      []string
	filteredItems []string
	selectedIndex int
	promptText    string
	input         string
	caseSensitive bool
	fontSize      int

	result string
	accept bool
}

func NewApp(items []string, prompt string, caseSensitive bool, fontSize int) *App {
	return &App{
		allItems:      append([]string(nil), items...),
		filteredItems: append([]string(nil), items...),
		selectedIndex: 0,
		promptText:    prompt,
		caseSensitive: caseSensitive,
		fontSize:      fontSize,
	}
}

func (a *App) startup(ctx context.Context) {
	a.ctx = ctx

	runtime.EventsOn(ctx, "frontend-ready", func(optionalData ...interface{}) {
		a.emitState()
	})
	runtime.EventsOn(ctx, "query-change", func(optionalData ...interface{}) {
		if len(optionalData) == 0 {
			return
		}
		query, ok := optionalData[0].(string)
		if !ok {
			return
		}
		a.input = query
		a.applyFilter(query)
		a.emitState()
	})
	runtime.EventsOn(ctx, "move-selection", func(optionalData ...interface{}) {
		if len(optionalData) == 0 {
			return
		}
		delta, ok := optionalData[0].(float64)
		if !ok {
			return
		}
		a.moveSelection(int(delta))
		a.emitState()
	})
	runtime.EventsOn(ctx, "click-selection", func(optionalData ...interface{}) {
		if len(optionalData) == 0 {
			return
		}
		idx, ok := optionalData[0].(float64)
		if !ok {
			return
		}
		a.selectedIndex = int(idx)
		a.acceptSelection()
	})
	runtime.EventsOn(ctx, "accept", func(optionalData ...interface{}) {
		a.acceptSelection()
	})
	runtime.EventsOn(ctx, "cancel", func(optionalData ...interface{}) {
		a.cancel()
	})
}

func (a *App) beforeClose(ctx context.Context) (prevent bool) {
	return false
}

func (a *App) emitState() {
	runtime.EventsEmit(a.ctx, "state", appState{
		PromptText:    a.promptText,
		Input:         a.input,
		FilteredItems: a.filteredItems,
		SelectedIndex: a.selectedIndex,
		FontSize:      a.fontSize,
	})
}

func (a *App) applyFilter(query string) {
	a.filteredItems = a.filteredItems[:0]
	q := query
	if !a.caseSensitive {
		q = strings.ToLower(q)
	}
	for _, item := range a.allItems {
		candidate := item
		if !a.caseSensitive {
			candidate = strings.ToLower(candidate)
		}
		if strings.Contains(candidate, q) {
			a.filteredItems = append(a.filteredItems, item)
		}
	}
	if len(a.filteredItems) == 0 {
		a.selectedIndex = -1
		return
	}
	a.selectedIndex = 0
}

func (a *App) moveSelection(delta int) {
	if len(a.filteredItems) == 0 {
		return
	}
	a.selectedIndex += delta
	if a.selectedIndex < 0 {
		a.selectedIndex = 0
	}
	if a.selectedIndex >= len(a.filteredItems) {
		a.selectedIndex = len(a.filteredItems) - 1
	}
}

func (a *App) acceptSelection() {
	if len(a.filteredItems) > 0 && a.selectedIndex >= 0 {
		a.result = a.filteredItems[a.selectedIndex]
	} else {
		a.result = strings.TrimSpace(a.input)
	}
	if a.result == "" {
		a.accept = false
		runtime.Quit(a.ctx)
		return
	}
	a.accept = true
	runtime.Quit(a.ctx)
}

func (a *App) cancel() {
	a.accept = false
	runtime.Quit(a.ctx)
}

package main

import (
	"bufio"
	"flag"
	"fmt"
	"io"
	"os"
	"strings"

	tk "modernc.org/tk9.0"
)

// appState contains all mutable UI state.
type appState struct {
	allItems      []string
	filteredItems []string
	selectedIndex int

	promptText string
	inputVar   string

	result string
	accept bool
}

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

	state := &appState{
		allItems:      items,
		filteredItems: append([]string(nil), items...),
		selectedIndex: 0,
		promptText:    *prompt,
	}

	if err := runUI(state, *caseSensitive, *fontSize); err != nil {
		fmt.Fprintf(os.Stderr, "ui error: %v\n", err)
		os.Exit(1)
	}

	if state.accept {
		fmt.Println(state.result)
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

func runUI(state *appState, caseSensitive bool, fontSize int) error {
	const maxVisibleItems = 12

	root := tk.App.With("className", "WhoaMenu")

	root.Configure(
		tk.Opt("padx", 6),
		tk.Opt("pady", 6),
	)
	root.WmTitle("whoamenu")
	root.WmAttributes("-topmost", 1)
	root.WmResizable(false, false)

	font := fmt.Sprintf("TkDefaultFont %d", fontSize)

	outer := tk.TFrame(root)
	outer.Pack(tk.FillX)

	prompt := tk.TLabel(outer, tk.Text(state.promptText), tk.Font(font))
	prompt.Pack(tk.SideLeft)

	entry := tk.TEntry(outer, tk.Textvariable(&state.inputVar), tk.Font(font), tk.Width(40))
	entry.Pack(tk.SideLeft, tk.FillX, tk.Expand)

	listHeight := len(state.filteredItems)
	if listHeight == 0 {
		listHeight = 1
	}
	if listHeight > maxVisibleItems {
		listHeight = maxVisibleItems
	}

	list := tk.Listbox(
		root,
		tk.Exportselection(0),
		tk.Height(listHeight),
		tk.Font(font),
	)
	list.Pack(tk.FillBoth, tk.Expand, tk.Pady("6 0"))

	renderList := func() {
		list.Delete(0, tk.End)
		for _, item := range state.filteredItems {
			list.Insert(tk.End, item)
		}
		if len(state.filteredItems) == 0 {
			state.selectedIndex = -1
			return
		}
		if state.selectedIndex < 0 {
			state.selectedIndex = 0
		}
		if state.selectedIndex >= len(state.filteredItems) {
			state.selectedIndex = len(state.filteredItems) - 1
		}
		list.SelectionSet(state.selectedIndex)
		list.Activate(state.selectedIndex)
		list.See(state.selectedIndex)
	}

	applyFilter := func(query string) {
		state.filteredItems = state.filteredItems[:0]
		q := query
		if !caseSensitive {
			q = strings.ToLower(q)
		}
		for _, item := range state.allItems {
			candidate := item
			if !caseSensitive {
				candidate = strings.ToLower(candidate)
			}
			if strings.Contains(candidate, q) {
				state.filteredItems = append(state.filteredItems, item)
			}
		}
		state.selectedIndex = 0
		renderList()
	}

	moveSelection := func(delta int) {
		if len(state.filteredItems) == 0 {
			return
		}
		state.selectedIndex += delta
		if state.selectedIndex < 0 {
			state.selectedIndex = 0
		}
		if state.selectedIndex >= len(state.filteredItems) {
			state.selectedIndex = len(state.filteredItems) - 1
		}
		list.SelectionClear(0, tk.End)
		list.SelectionSet(state.selectedIndex)
		list.Activate(state.selectedIndex)
		list.See(state.selectedIndex)
	}

	acceptSelection := func() {
		if len(state.filteredItems) > 0 && state.selectedIndex >= 0 {
			state.result = state.filteredItems[state.selectedIndex]
		} else {
			state.result = strings.TrimSpace(state.inputVar)
		}
		if state.result == "" {
			state.accept = false
			root.Destroy()
			return
		}
		state.accept = true
		root.Destroy()
	}

	cancel := func() {
		state.accept = false
		root.Destroy()
	}

	entry.Bind("<KeyRelease>", func(_ tk.Event) { applyFilter(state.inputVar) })
	entry.Bind("<Down>", func(_ tk.Event) { moveSelection(1) })
	entry.Bind("<Up>", func(_ tk.Event) { moveSelection(-1) })
	entry.Bind("<Return>", func(_ tk.Event) { acceptSelection() })
	entry.Bind("<Escape>", func(_ tk.Event) { cancel() })

	list.Bind("<Double-Button-1>", func(_ tk.Event) {
		idx := list.Curselection()
		if len(idx) == 0 {
			return
		}
		state.selectedIndex = idx[0]
		acceptSelection()
	})

	list.Bind("<ButtonRelease-1>", func(_ tk.Event) {
		idx := list.Curselection()
		if len(idx) == 0 {
			return
		}
		state.selectedIndex = idx[0]
	})

	renderList()
	entry.Focus()

	root.Bind("<Escape>", func(_ tk.Event) { cancel() })
	root.Bind("<Return>", func(_ tk.Event) { acceptSelection() })

	root.Center().Wait()
	return nil
}

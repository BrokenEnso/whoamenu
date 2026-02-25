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

	root := tk.App

	root.Configure(tk.Padx(6), tk.Pady(6))
	root.WmTitle("whoamenu")
	tk.WmAttributes(root, "-topmost", 1)
	root.SetResizable(false, false)

	font := fmt.Sprintf("TkDefaultFont %d", fontSize)

	outer := root.TFrame()
	tk.Pack(outer, tk.Fill(tk.FILL_X))

	prompt := outer.TLabel(tk.Txt(state.promptText), tk.Font(font))
	tk.Pack(prompt, tk.Side(tk.LEFT))

	entry := outer.TEntry(tk.Textvariable("inputVar"), tk.Font(font), tk.Width(40))
	tk.Pack(entry, tk.Side(tk.LEFT), tk.Fill(tk.FILL_X), tk.Expand(1))

	listHeight := len(state.filteredItems)
	if listHeight == 0 {
		listHeight = 1
	}
	if listHeight > maxVisibleItems {
		listHeight = maxVisibleItems
	}

	var list *tk.ListboxWidget
	var acceptSelection func()

	createList := func() {
		if list != nil {
			tk.Destroy(list)
		}
		list = root.Listbox(
			tk.Exportselection(0),
			tk.Height(listHeight),
			tk.Font(font),
		)
		tk.Pack(list, tk.Fill(tk.FILL_BOTH), tk.Expand(1), tk.Pady("6 0"))

		tk.Bind(list, "<Double-Button-1>", tk.Command(func() {
			idx := list.Curselection()
			if len(idx) == 0 {
				return
			}
			state.selectedIndex = idx[0]
			acceptSelection()
		}))

		tk.Bind(list, "<ButtonRelease-1>", tk.Command(func() {
			idx := list.Curselection()
			if len(idx) == 0 {
				return
			}
			state.selectedIndex = idx[0]
		}))
	}

	createList()

	renderList := func() {
		createList()
		for _, item := range state.filteredItems {
			list.Insert(tk.END, item)
		}
		if len(state.filteredItems) == 0 {
			state.selectedIndex = -1
		} else {
			if state.selectedIndex < 0 {
				state.selectedIndex = 0
			}
			if state.selectedIndex >= len(state.filteredItems) {
				state.selectedIndex = len(state.filteredItems) - 1
			}
		}
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
	}

	acceptSelection = func() {
		if len(state.filteredItems) > 0 && state.selectedIndex >= 0 {
			state.result = state.filteredItems[state.selectedIndex]
		} else {
			state.result = strings.TrimSpace(state.inputVar)
		}
		if state.result == "" {
			state.accept = false
			tk.Destroy(root)
			return
		}
		state.accept = true
		tk.Destroy(root)
	}

	cancel := func() {
		state.accept = false
		tk.Destroy(root)
	}

	tk.Bind(entry, "<KeyRelease>", tk.Command(func() {
		state.inputVar = entry.Textvariable()
		applyFilter(state.inputVar)
	}))
	tk.Bind(entry, "<Down>", tk.Command(func() { moveSelection(1) }))
	tk.Bind(entry, "<Up>", tk.Command(func() { moveSelection(-1) }))
	tk.Bind(entry, "<Return>", tk.Command(func() { acceptSelection() }))
	tk.Bind(entry, "<Escape>", tk.Command(func() { cancel() }))

	renderList()
	tk.Focus(entry)

	tk.Bind(root, "<Escape>", tk.Command(func() { cancel() }))
	tk.Bind(root, "<Return>", tk.Command(func() { acceptSelection() }))

	root.Center().Wait()
	return nil
}

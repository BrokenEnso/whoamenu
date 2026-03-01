using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Layout;
using Avalonia.Media;

namespace WhoaMenu;

public class MainWindow : Window
{
    private readonly IReadOnlyList<string> _allItems;
    private readonly bool _caseSensitive;
    private readonly TextBox _input;
    private readonly ListBox _list;

    internal MainWindow(IReadOnlyList<string> items, CliOptions options)
    {
        _allItems = items;
        _caseSensitive = options.CaseSensitive;
        
        Width = 720;
        Height = CalculateWindowHeight(options.Lines, options.FontSize);
        CanResize = false;
        Topmost = true;
        SystemDecorations = SystemDecorations.None;

        var root = new DockPanel();

        var header = new DockPanel
        {
            LastChildFill = true,
            //Margin = new Thickness(8)
        };

        var promptBlock = new TextBlock
        {
            Text = options.Prompt,
            FontSize = options.FontSize,
            VerticalAlignment = VerticalAlignment.Center,
            [DockPanel.DockProperty] = Dock.Left
        };

        _input = new TextBox
        {
            FontSize = options.FontSize,
            HorizontalAlignment = HorizontalAlignment.Stretch,
            BorderThickness = new Thickness(0),
        };
        //_input.Classes.
        //_input.se
        _input.AttachedToVisualTree += (_, _) => _input.Focus();
        _input.KeyDown += HandleInputKeyDown;
        _input.TextChanged += (_, _) => ApplyFilter();
        header.Children.Add(promptBlock);
        header.Children.Add(_input);

        DockPanel.SetDock(header, Dock.Top);
        root.Children.Add(header);

        _list = new ListBox
        {
            //Margin = new Thickness(8, 0, 8, 8),
            FontSize = options.FontSize,
            ItemsSource = _allItems
        };
        _list.DoubleTapped += (_, _) => AcceptSelection();
        root.Children.Add(_list);

        Content = root;

        ApplyFilter();
    }

    private void HandleInputKeyDown(object? sender, KeyEventArgs e)
    {
        switch (e.Key)
        {
            case Key.Down:
                MoveSelection(1);
                e.Handled = true;
                break;
            case Key.Up:
                MoveSelection(-1);
                e.Handled = true;
                break;
            case Key.Enter:
                AcceptSelection();
                e.Handled = true;
                break;
            case Key.Escape:
                CancelSelection();
                e.Handled = true;
                break;
        }
    }

    private void MoveSelection(int delta)
    {
        if (_list.ItemCount == 0)
        {
            return;
        }

        var next = _list.SelectedIndex + delta;
        if (next < 0)
        {
            next = 0;
        }
        else if (next >= _list.ItemCount)
        {
            next = _list.ItemCount - 1;
        }

        _list.SelectedIndex = next;
        if (_list.SelectedItem != null)
        {
            _list.ScrollIntoView(_list.SelectedItem);
        }
    }

    private void ApplyFilter()
    {
        var query = _input.Text ?? string.Empty;
        var filtered = _allItems.Where(item => Matches(item, query)).ToList();

        _list.ItemsSource = filtered;
        _list.SelectedIndex = filtered.Count > 0 ? 0 : -1;
    }

    private bool Matches(string item, string query)
    {
        if (_caseSensitive)
        {
            return item.Contains(query);
        }

        return item.Contains(query, StringComparison.OrdinalIgnoreCase);
    }

    private void AcceptSelection()
    {
        var result = _list.SelectedItem as string;
        if (string.IsNullOrWhiteSpace(result))
        {
            result = (_input.Text ?? string.Empty).Trim();
        }

        Session.Accepted = !string.IsNullOrWhiteSpace(result);
        Session.Result = result ?? string.Empty;
        Close();
    }


    private static double CalculateWindowHeight(int lines, int fontSize)
    {
        var lineCount = Math.Max(1, lines);
        var rowHeight = Math.Max(18, fontSize + 10);
        const int headerHeight = 44;
        const int verticalPadding = 12;

        return headerHeight + verticalPadding + (lineCount * rowHeight);
    }
    private void CancelSelection()
    {
        Session.Accepted = false;
        Session.Result = string.Empty;
        Close();
    }
}

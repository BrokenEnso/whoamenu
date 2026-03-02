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
        Topmost = true;
        SystemDecorations = SystemDecorations.None;

        if (options.NormalBackground is { } normalBackground)
        {
            Background = new SolidColorBrush(normalBackground);
        }

        var root = new DockPanel();

        if (options.NormalBackground is { } rootBackground)
        {
            root.Background = new SolidColorBrush(rootBackground);
        }

        var header = new DockPanel
        {
            LastChildFill = true,
        };

        if (options.NormalBackground is { } headerBackground)
        {
            header.Background = new SolidColorBrush(headerBackground);
        }

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

        if (options.NormalBackground is { } inputBackground)
        {
            _input.Background = new SolidColorBrush(inputBackground);
        }
        
        _input.AttachedToVisualTree += (_, _) => _input.Focus();
        _input.KeyDown += HandleInputKeyDown;
        _input.TextChanged += (_, _) => ApplyFilter();

        header.Children.Add(promptBlock);
        header.Children.Add(_input);

        DockPanel.SetDock(header, Dock.Top);
        root.Children.Add(header);

        _list = new ListBox
        {
            FontSize = options.FontSize,
            ItemsSource = _allItems
        };

        if (options.NormalBackground is { } listBackground)
        {
            _list.Background = new SolidColorBrush(listBackground);
        }
        _list.DoubleTapped += (_, _) => AcceptSelection();
        root.Children.Add(_list);

        Content = root;

        //Calculating the height(text area + items to display) after the actual item heights are avialable
        Loaded += (_, _) =>
        {
            if(_list.ContainerFromIndex(0) is ListBoxItem item)
            {
                Height = _input.Bounds.Height + (item.Bounds.Height * options.Lines);
            }

            var monitorIndex = Math.Clamp(Session.Options.Monitor, 0, Screens.All.Count - 1);
            var target = Screens.All[monitorIndex];
            var workingArea = target.WorkingArea;
            var windowWidth = (int)Math.Round(Width * target.Scaling);
            var windowHeight = (int)Math.Round(Height * target.Scaling);

            var x = workingArea.X + (workingArea.Width - windowWidth) / 2;
            var y = workingArea.Y + (workingArea.Height - windowHeight) / 2;
            if (Session.Options.Bottom)
            {
                y = workingArea.Bottom - windowHeight;
            }
            if (Session.Options.Top)
            {
                y = workingArea.Y;
            }

            Console.Error.WriteLine( $"X: {x} Y: {y}");

            Position = new PixelPoint(x, y);
        };

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

    private void CancelSelection()
    {
        Session.Accepted = false;
        Session.Result = string.Empty;
        Close();
    }
}

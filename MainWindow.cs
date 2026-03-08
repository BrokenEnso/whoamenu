using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Layout;
using Avalonia.Media;
using Avalonia.Media.Fonts;
using Avalonia.Threading;

namespace WhoaMenu;

public class MainWindow : Window
{
    private readonly IReadOnlyList<string> _allItems;
    private readonly bool _caseSensitive;
    private readonly TextBox _input;
    private readonly ListBox _list;
    private bool _hasScheduledStartupFocusRetry;

    internal MainWindow(IReadOnlyList<string> items, CliOptions options)
    {
        _allItems = items;
        _caseSensitive = options.CaseSensitive;
        var fontFamily = string.IsNullOrWhiteSpace(options.FontName) ? FontFamily.Default : new FontFamily(options.FontName);
        
        Width = 720;
        Topmost = true;
        SystemDecorations = SystemDecorations.None;

        var root = new DockPanel();

        var header = new DockPanel
        {
            LastChildFill = true,
        };

        var promptBlock = new TextBlock
        {
            Text = options.Prompt,
            FontSize = options.FontSize,
            FontFamily = fontFamily,
            VerticalAlignment = VerticalAlignment.Center,
            [DockPanel.DockProperty] = Dock.Left,
        };

        _input = new TextBox
        {
            FontSize = options.FontSize,
            FontFamily = fontFamily,
            HorizontalAlignment = HorizontalAlignment.Stretch,
            BorderThickness = new Thickness(0),
        };

        _input.TextChanged += (_, _) => ApplyFilter();

        KeyDown += HandleInputKeyDown;
        Opened += (_, _) => EnsureInputFocus(scheduleRetry: true);
        Activated += (_, _) => EnsureInputFocus();

        header.Children.Add(promptBlock);
        header.Children.Add(_input);

        DockPanel.SetDock(header, Dock.Top);
        root.Children.Add(header);

        _list = new ListBox
        {
            FontSize = options.FontSize,
            FontFamily = fontFamily,
            ItemsSource = _allItems
        };


        if (options.NormalForeground is { } normalForground)
        {
            promptBlock.Foreground = new SolidColorBrush(normalForground);
            _input.Foreground = new SolidColorBrush(normalForground);
            _input.Resources["TextControlForegroundPointerOver"] = new SolidColorBrush(normalForground);
            _input.Resources["TextControlForegroundFocused"] = new SolidColorBrush(normalForground);
            _list.Foreground = new SolidColorBrush(normalForground);
        }

        if (options.NormalBackground is { } normalBackground)
        {
            Background = new SolidColorBrush(normalBackground);
            root.Background = new SolidColorBrush(normalBackground);
            header.Background = new SolidColorBrush(normalBackground);
            _input.Background = new SolidColorBrush(normalBackground);
            _input.Resources["TextControlBackgroundPointerOver"] = new SolidColorBrush(normalBackground);
            _input.Resources["TextControlBackgroundFocused"] = new SolidColorBrush(normalBackground);
            _list.Background = new SolidColorBrush(normalBackground);
            _list.Resources["Background"] = new SolidColorBrush(normalBackground);
            _list.Resources["SystemControlHighlightListLowBrush"] = new SolidColorBrush(normalBackground);
        }


        if(options.SelectedForeground is { } selectedForeground)
        {
            _list.Resources["SystemControlHighlightAltBaseHighBrush"] = new SolidColorBrush(selectedForeground);
        }

        if (options.SelectedBackground is { } selectedBackground)
        {
            _list.Resources["SystemControlHighlightListAccentLowBrush"] = new SolidColorBrush(selectedBackground);
            _list.Resources["SystemControlHighlightListAccentHighBrush"] = new SolidColorBrush(selectedBackground);
            _list.Resources["SystemControlHighlightListMediumBrush"] = new SolidColorBrush(selectedBackground);
            _list.Resources["SystemControlHighlightListAccentMediumBrush"] = new SolidColorBrush(selectedBackground);
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

            Position = new PixelPoint(x, y);
            ApplyFilter();
            EnsureInputFocus();
        }; 
    }

    private void EnsureInputFocus(bool scheduleRetry = false)
    {
        Activate();
        _input.Focus();

        if (!scheduleRetry || _hasScheduledStartupFocusRetry)
        {
            return;
        }

        _hasScheduledStartupFocusRetry = true;

        DispatcherTimer.RunOnce(() =>
        {
            Activate();
            _input.Focus();
        }, TimeSpan.FromMilliseconds(75));
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

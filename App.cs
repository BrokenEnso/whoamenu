using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Markup.Xaml;

namespace WhoaMenu;

public class App : Application
{
    public override void Initialize()
    {
        AvaloniaXamlLoader.Load(this);
    }

    public override void OnFrameworkInitializationCompleted()
    {
        if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
        {
            desktop.MainWindow = new MainWindow(Session.Items, Session.Options)
            {
                WindowStartupLocation = WindowStartupLocation.Manual,
            };

            var screens = desktop.MainWindow.Screens.All;
            var monitorIndex = Math.Clamp(Session.Options.Monitor, 0, screens.Count - 1);
            var target = screens[monitorIndex];
            var workingArea = target.WorkingArea;

            var windowWidth = (int)Math.Round(desktop.MainWindow.Width * target.Scaling);
            var windowHeight = (int)Math.Round(desktop.MainWindow.Height * target.Scaling);

            var x = workingArea.X + (workingArea.Width - windowWidth) / 2;
            var y = Session.Options.Bottom
                ? workingArea.Bottom - windowHeight
                : workingArea.Y;

            desktop.MainWindow.Position = new PixelPoint(x, y);
        }

        base.OnFrameworkInitializationCompleted();
    }
}

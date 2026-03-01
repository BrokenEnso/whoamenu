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
            var wa = target.WorkingArea;

            var x = wa.X + (wa.Width - (int)desktop.MainWindow.Width) / 2;
            var y = Session.Options.Bottom
                ? wa.Y + wa.Height - (int)desktop.MainWindow.Height
                : wa.Y;

            desktop.MainWindow.Position = new PixelPoint(x, y);
        }

        base.OnFrameworkInitializationCompleted();
    }
}

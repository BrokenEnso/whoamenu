using Avalonia;
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
                WindowStartupLocation = Avalonia.Controls.WindowStartupLocation.CenterScreen,
            };

            var screens = desktop.MainWindow.Screens.All;
            var target = screens.Count > Session.Options.Monitor ? screens[Session.Options.Monitor] : desktop.MainWindow.Screens.Primary ?? screens[0]; // fallback
            var wa = target.WorkingArea;
            desktop.MainWindow.Position = new PixelPoint(wa.X + 50, wa.Y + 50);
        }

        base.OnFrameworkInitializationCompleted();
    }
}

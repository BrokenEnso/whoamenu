using System.Security.AccessControl;
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
                WindowStartupLocation = WindowStartupLocation.Manual
            };
        }

        base.OnFrameworkInitializationCompleted();
    }
}

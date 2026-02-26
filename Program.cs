using System.Text;
using Avalonia;

namespace WhoaMenu;

internal static class Program
{
    [STAThread]
    public static int Main(string[] args)
    {
        var options = CliOptions.Parse(args);
        var items = ReadItems(Console.In);

        Session.Options = options;
        Session.Items = items;

        BuildAvaloniaApp().StartWithClassicDesktopLifetime(args);

        if (Session.Accepted && !string.IsNullOrWhiteSpace(Session.Result))
        {
            Console.OutputEncoding = Encoding.UTF8;
            Console.WriteLine(Session.Result);
            return 0;
        }

        return 1;
    }

    public static AppBuilder BuildAvaloniaApp()
        => AppBuilder.Configure<App>()
            .UsePlatformDetect();

    private static List<string> ReadItems(TextReader reader)
    {
        var items = new List<string>();
        string? line;

        while ((line = reader.ReadLine()) is not null)
        {
            line = line.Trim();
            if (!string.IsNullOrEmpty(line))
            {
                items.Add(line);
            }
        }

        return items;
    }
}

internal sealed record CliOptions(string Prompt, bool CaseSensitive, int FontSize)
{
    public static CliOptions Parse(string[] args)
    {
        var prompt = ">";
        var caseSensitive = false;
        var fontSize = 12;

        for (var i = 0; i < args.Length; i++)
        {
            switch (args[i])
            {
                case "-p" when i + 1 < args.Length:
                    prompt = args[++i];
                    break;
                case "-case-sensitive":
                    caseSensitive = true;
                    break;
                case "-font-size" when i + 1 < args.Length && int.TryParse(args[++i], out var parsed):
                    fontSize = parsed;
                    break;
            }
        }

        return new CliOptions(prompt, caseSensitive, fontSize);
    }
}

internal static class Session
{
    public static CliOptions Options { get; set; } = new(">", false, 12);
    public static IReadOnlyList<string> Items { get; set; } = Array.Empty<string>();
    public static bool Accepted { get; set; }
    public static string Result { get; set; } = string.Empty;
}

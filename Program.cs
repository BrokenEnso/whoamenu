using System.Text;
using Avalonia;
using Avalonia.Media;

namespace WhoaMenu;

internal static class Program
{
    [STAThread]
    public static int Main(string[] args)
    {
        var options = CliOptions.Parse(args);
        var items = ReadItems(Console.In);

        if(items.Count == 0)
        {
            Console.Error.WriteLine("No items provided");
            return 1;
        }

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

internal sealed record CliOptions(
    string Prompt,
    bool CaseSensitive,
    int FontSize,
    int Monitor,
    bool Bottom,
    bool Top,
    int Lines,
    Color? NormalBackground)
{
    public static CliOptions Parse(string[] args)
    {
        var prompt = ">";
        var caseSensitive = false;
        var fontSize = 12;
        var monitor = 0;
        var bottom = false;
        var top = false;
        var lines = 10;
        Color? normalBackground = null;

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
                case "-m" when i + 1 < args.Length && int.TryParse(args[++i], out var parsed):
                    monitor = parsed - 1; //ajusting form the logical to array index
                    break;
                case "-b":
                    bottom = true;
                    break;
                case "-t":
                    top = true;
                    break;
                case "-l" when i + 1 < args.Length && int.TryParse(args[++i], out var parsed):
                    lines = Math.Max(1, parsed);
                    break;
                case "-nb" when i + 1 < args.Length:
                    var colorText = args[++i];
                    if (!Color.TryParse(colorText, out var parsedColor))
                    {
                        Console.Error.WriteLine($"Invalid color for -nb: '{colorText}'");
                        Environment.Exit(1);
                    }

                    normalBackground = parsedColor;
                    break;
            }
        }

        return new CliOptions(prompt, caseSensitive, fontSize, monitor, bottom, top, lines, normalBackground);
    }
}

internal static class Session
{
    public static CliOptions Options { get; set; } = new(">", false, 12, 0, false, false, 10, null);
    public static IReadOnlyList<string> Items { get; set; } = Array.Empty<string>();
    public static bool Accepted { get; set; }
    public static string Result { get; set; } = string.Empty;
}

using System.Text;
using System.Globalization;
using Avalonia;
using Avalonia.Media;

namespace WhoaMenu;

internal static class Program
{
    [STAThread]
    public static int Main(string[] args)
    {
        var configArgs = ConfigFile.TryLoadArgs();
        var mergedArgs = configArgs.Concat(args).ToArray();
        var options = CliOptions.Parse(mergedArgs);

        if (options.ShowHelp)
        {
            Console.WriteLine(CliOptions.UsageText);
            return 0;
        }

        Session.InputPiped = Console.IsInputRedirected; //If no piped input then collect text input
        
        List<string> items = new List<string>();

        if (Session.InputPiped)
        {
            items = ReadItems(Console.In);
        }

        if(Session.InputPiped && items.Count == 0)
        {
            Console.Error.WriteLine("No items provided");
            return 1;
        }

        Session.Items = items;
        Session.Options = options;
        
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

internal static class ConfigFile
{
    public static IReadOnlyList<string> TryLoadArgs()
    {
        var path = GetPath();
        if (!File.Exists(path))
        {
            return Array.Empty<string>();
        }

        var args = new List<string>();

        foreach (var rawLine in File.ReadLines(path))
        {
            var line = rawLine.Trim();
            if (string.IsNullOrWhiteSpace(line) || line.StartsWith('#'))
            {
                continue;
            }

            args.AddRange(Tokenize(line));
        }

        return args;
    }

    private static string GetPath()
    {
        var xdgConfigHome = Environment.GetEnvironmentVariable("XDG_CONFIG_HOME");
        if (!string.IsNullOrWhiteSpace(xdgConfigHome))
        {
            return Path.Combine(xdgConfigHome, "whoamenu", "config");
        }

        var home = Environment.GetEnvironmentVariable("HOME")
            ?? Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);

        return Path.Combine(home, ".config", "whoamenu", "config");
    }

    private static IEnumerable<string> Tokenize(string line)
    {
        var token = new StringBuilder();
        var inQuotes = false;

        for (var i = 0; i < line.Length; i++)
        {
            var current = line[i];

            if (current == '"')
            {
                inQuotes = !inQuotes;
                continue;
            }

            if (!inQuotes && char.IsWhiteSpace(current))
            {
                if (token.Length > 0)
                {
                    yield return token.ToString();
                    token.Clear();
                }

                continue;
            }

            token.Append(current);
        }

        if (token.Length > 0)
        {
            yield return token.ToString();
        }
    }
}

internal sealed record CliOptions(
    bool ShowHelp,
    string Prompt,
    bool CaseSensitive,
    int FontSize,
    string? FontName,
    int Monitor,
    bool Bottom,
    bool Top,
    int Lines,
    double? CornerRadius,
    double? Transparency,
    Color? NormalBackground,
    Color? NormalForeground,
    Color? SelectedBackground,
    Color? SelectedForeground)
{
    public static CliOptions Parse(string[] args)
    {
        var showHelp = false;
        var prompt = ">";
        var caseSensitive = false;
        var fontSize = 12;
        string? fontName = null;
        var monitor = 0;
        var bottom = false;
        var top = false;
        var lines = 10;
        double? cornerRadius = null;
        double? transparency = null;
        Color? normalBackground = null;
        Color? normalForeground = null;
        Color? selectedBackground = null;
        Color? selectedForeground = null;

        for (var i = 0; i < args.Length; i++)
        {
            switch (args[i])
            {
                case "-h":
                    showHelp = true;
                    break;
                case "-p" when i + 1 < args.Length:
                    prompt = args[++i];
                    break;
                case "-case-sensitive":
                    caseSensitive = true;
                    break;
                case "-font-size" when i + 1 < args.Length && int.TryParse(args[++i], out var parsed):
                    fontSize = parsed;
                    break;
                case "-fn" when i + 1 < args.Length:
                    fontName = args[++i];
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
                case "-rc":
                    if (i + 1 < args.Length && double.TryParse(args[i + 1], NumberStyles.Float, CultureInfo.InvariantCulture, out var parsedRadius))
                    {
                        cornerRadius = Math.Clamp(parsedRadius, 0, 30); //Using 30 as upper bound, but is likely too high
                        i++;
                    }

                    break;
                case "-tr" when i + 1 < args.Length:
                    var transparencyText = args[++i];
                    if (!double.TryParse(transparencyText, NumberStyles.Float, CultureInfo.InvariantCulture, out var parsedTransparency))
                    {
                        Console.Error.WriteLine($"Invalid transparency for -tr: '{transparencyText}'");
                        Environment.Exit(1);
                    }

                    transparency = Math.Clamp(parsedTransparency, 0, 1);
                    break;
                case "-nb" when i + 1 < args.Length:
                    var colorText = args[++i];
                    if (!TryParseColor(colorText, out var parsedColor))
                    {
                        Console.Error.WriteLine($"Invalid color for -nb: '{colorText}'");
                        Environment.Exit(1);
                    }

                    normalBackground = parsedColor;
                    break;
                case "-nf" when i + 1 < args.Length:
                    colorText = args[++i];
                    if (!TryParseColor(colorText, out parsedColor))
                    {
                        Console.Error.WriteLine($"Invalid color for -nf: '{colorText}'");
                        Environment.Exit(1);
                    }

                    normalForeground = parsedColor;
                    break;
                case "-sb" when i + 1 < args.Length:
                    colorText = args[++i];
                    if (!TryParseColor(colorText, out parsedColor))
                    {
                        Console.Error.WriteLine($"Invalid color for -sb: '{colorText}'");
                        Environment.Exit(1);
                    }

                    selectedBackground = parsedColor;
                    break;
                case "-sf" when i + 1 < args.Length:
                    colorText = args[++i];
                    if (!TryParseColor(colorText, out parsedColor))
                    {
                        Console.Error.WriteLine($"Invalid color for -sf: '{colorText}'");
                        Environment.Exit(1);
                    }

                    selectedForeground = parsedColor;
                    break;
            }
        }

        return new CliOptions(
            showHelp,
            prompt,
            caseSensitive,
            fontSize,
            fontName,
            monitor,
            bottom,
            top,
            lines,
            cornerRadius,
            transparency,
            normalBackground,
            normalForeground,
            selectedBackground,
            selectedForeground);
    }

    public const string UsageText =
        "-h\tshows this usage message and exits.\n" +
        "Configuration is loaded from $XDG_CONFIG_HOME/whoamenu/config (or $HOME/.config/whoamenu/config).\n" +
        "Command line flags override configuration file values.\n" +
        "-p <prompt>\tdefines a prompt to be displayed before the input area.\n" +
        "-case-sensitive\tmakes matching case sensitive.\n" +
        "-font-size <size>\tdefines the font size.\n" +
        "-fn <font>\tdefines the font.\n" +
        "-m <monitor>\tdefines the target monitor index (1-based).\n" +
        "-b\tdefines that menu appears at the bottom.\n" +
        "-t\tdefines that menu appears at the top.\n" +
        "-l <lines>\tactivates vertical list mode with the given number of lines.\n" +
        "-rc [radius]\tsets window corner radius; omit value to leave unchanged.\n" +
        "-tr <0-1>\tsets the window opacity/transparency level.\n" +
        "-nb <color>\tdefines the normal background color (#RGB, #RRGGBB, and color names are supported).\n" +
        "-nf <color>\tdefines the normal foreground color (#RGB, #RRGGBB, and color names are supported).\n" +
        "-sb <color>\tdefines the selected background color (#RGB, #RRGGBB, and color names are supported).\n" +
        "-sf <color>\tdefines the selected foreground color (#RGB, #RRGGBB, and color names are supported).";

    private static bool TryParseColor(string value, out Color color)
    {
        string norValue = (value.Length == 3 || value.Length == 6) ? $"#{value}" : value;  

        if (Color.TryParse(norValue, out color))
        {
            return true;
        }

        return false;
    }
}

internal static class Session
{
    public static CliOptions Options { get; set; } = new(false, ">", false, 12, null, 0, false, false, 10, null, null, null, null, null, null);
    public static IReadOnlyList<string> Items { get; set; } = Array.Empty<string>();
    public static bool InputPiped { get; set; }
    public static bool Accepted { get; set; }
    public static string Result { get; set; } = string.Empty;
}

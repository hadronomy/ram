using System.Collections.Generic;
using System.CommandLine.Help;
using System.CommandLine;
using System.Linq;
using Spectre.Console;
using System.CommandLine.Invocation;

namespace MemoryMachine.Console.Extensions
{
    public static class HelpExtensions
    {
        public static HelpBuilder CustomizeHelpLayout(this HelpBuilder helpBuilder, RootCommand rootCommand)
        {
            helpBuilder.CustomizeLayout(_ =>
                new List<HelpSectionDelegate>()
                    .Append(ctx =>
                    {
                        if (!ctx.Command.Parents.Any())
                        {
                            AnsiConsole.Write(new FigletText(ctx.Command.Name));
                        }
                    })
                    .Append(ctx => AnsiConsole.MarkupLine($"{ctx.Command.Description}"))
                    .Append(ctx =>
                    {
                        var commandPath = new Stack<string>();
                        var current = ctx.Command;

                        while (current != null)
                        {
                            commandPath.Push(current.Name);
                            current = current.Parents.FirstOrDefault() as Command;
                        }

                        var hasSubcommands = ctx.Command.Children.OfType<Command>().Any();
                        var usage = $"Usage: [blue]{string.Join(" ", commandPath)}[/]";

                        if (hasSubcommands)
                        {
                            usage += " <command>";
                        }

                        usage += " [teal][[...flags]][/] [[...args]]";
                        AnsiConsole.MarkupLine(usage);
                    })
                    .Append(ctx =>
                    {
                        if (ctx.Command.Children.OfType<Command>().Any())
                        {
                            var table = new Table().BorderColor(Color.Grey);
                            table.AddColumn("Commands");
                            table.AddColumn("Description");
                            table.AddColumn("Example");

                            foreach (var cmd in ctx.Command.Children.OfType<Command>())
                            {
                                table.AddRow(
                                    $"[fuchsia]{cmd.Name}[/]",
                                    cmd.Description ?? "",
                                    $"{rootCommand.Name} {cmd.Name} --help"
                                );
                            }
                            AnsiConsole.Write(table);
                        }
                    })
                    .Append(ctx =>
                    {
                        var table = new Table().BorderColor(Color.Grey);
                        table.AddColumn("Options");
                        table.AddColumn("Description");

                        foreach (var option in ctx.Command.Options)
                        {
                            var aliases = string.Join(", ", option.Aliases.Select(a => $"[teal]{a}[/]"));
                            table.AddRow(aliases, option.Description ?? "");
                        }
                        AnsiConsole.Write(table);
                    })
            );
            return helpBuilder;
        }
    }
}

using MemoryMachine.Console.Extensions;
using System.CommandLine;
using System.CommandLine.Builder;
using Spectre.Console;
using System.Threading.Tasks;
using System.CommandLine.Parsing;
using System.CommandLine.Help;
using System.Linq;

namespace MemoryMachine.Console
{
    public static class Program
    {
        public static async Task<int> Main(string[] args)
        {
            try
            {
                var rootCommand = new RootCommand("Memory Machine emulator")
                {
                    Name = "mm",
                    Description = "A tool emulating a memory machine with a custom set of instructions."
                };

                var parser = new CommandLineBuilder(rootCommand)
                    .AddCommands()
                    .UseDefaults()
                    .UseHelp(ctx => ctx.HelpBuilder.CustomizeHelpLayout(rootCommand))
                    .UseExceptionHandler((ex, context) =>
                    {
                        AnsiConsole.MarkupLine($"[red]Error:[/] {ex.Message}");
                        context.ExitCode = 1;
                    })
                    .Build();

                return await parser.InvokeAsync(args);
            }
            catch (System.Exception ex)
            {
                AnsiConsole.WriteException(ex);
                return 1;
            }
        }
    }
}

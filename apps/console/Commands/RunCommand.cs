using System.CommandLine;
using Spectre.Console;
using MemoryMachine.Core;
using System.Diagnostics;
using MemoryMachine.Core.Instructions;

namespace MemoryMachine.Console.Commands
{
    using MemoryMachine = MemoryMachine.Core.MemoryMachine;

    public class RunCommand : Command
    {
        public RunCommand() : base("run", "Executes one or more Memory Machine programs and compares their performance.")
        {
            var filePathsArgument = new Argument<string[]>("files", "The paths to the Memory Machine program files.");
            AddArgument(filePathsArgument);

            this.SetHandler(ExecuteRunCommands, filePathsArgument);
        }

        private void ExecuteRunCommands(string[] files)
        {
            if (files == null || files.Length == 0)
            {
                AnsiConsole.MarkupLine("[red]No files specified.[/]");
                return;
            }

            List<(string FileName, long ExecutionTime, int InstructionsExecuted, double InstructionsPerSecond, List<int> Output)> results = [];

            foreach (var file in files)
            {
                AnsiConsole.MarkupLine($"[green]Executing program:[/] {file}");

                try
                {
                    var instructionSet = new InstructionSet();
                    var assemblyLoader = new AssemblyLoader(instructionSet);
                    var assembly = assemblyLoader.LoadFromFile(file);

                    var machine = new MemoryMachine();
                    machine.LoadAssembly(assembly);

                    AnsiConsole.MarkupLine("[yellow]Enter input values (separated by spaces, enter 'done' when finished):[/]");
                    var input = AnsiConsole.Ask<string>("[grey]> [/]");
                    var inputValues = input.Split(' ', StringSplitOptions.RemoveEmptyEntries)
                        .Select(s => int.TryParse(s, out var n) ? n : (int?)null)
                        .Where(n => n.HasValue)
                        .Select(n => n!.Value)
                        .ToList();

                    machine.QueueInputRange(inputValues);

                    var stopwatch = Stopwatch.StartNew();
                    machine.Execute();
                    stopwatch.Stop();

                    AnsiConsole.WriteLine();
                    AnsiConsole.MarkupLine("[green]Output:[/]");
                    var output = string.Join(", ", machine.OutputBuffer);
                    AnsiConsole.MarkupLine($"[yellow]{output}[/]");

                    results.Add((file, stopwatch.ElapsedMilliseconds, machine.ExecutedInstructions, machine.ExecutedInstructions / (stopwatch.ElapsedMilliseconds / 1000.0), machine.OutputBuffer.ToList()));
                }
                catch (AssemblyLoadException ex)
                {
                    AnsiConsole.WriteLine();
                    AnsiConsole.MarkupLine($"[red]Error loading assembly: {ex.Message}[/]");
                    return;
                }
                catch (Exception ex)
                {
                    AnsiConsole.WriteException(ex);
                }

                AnsiConsole.WriteLine();
            }

            var bestResult = results.OrderBy(r => r.ExecutionTime).FirstOrDefault();
            if (bestResult == default)
            {
                AnsiConsole.MarkupLine("[red]No program could be executed successfully.[/]");
                return;
            }

            if (files.Length > 1)
            {
                AnsiConsole.WriteLine();
                AnsiConsole.MarkupLine($"[green]Best program:[/] {bestResult.FileName}");
            }

            DisplayResultTable(bestResult, files.Length > 1);
        }

        private static void DisplayResultTable((string FileName, long ExecutionTime, int InstructionsExecuted, double InstructionsPerSecond, List<int> Output) result, bool isComparison)
        {
            var table = new Table();
            table.AddColumn(new TableColumn("[bold blue]Metric[/]"));
            table.AddColumn(new TableColumn("[bold blue]Value[/]"));

            string metricColor = isComparison ? "[blue]" : "[yellow]";
            string valueColor = isComparison ? "[yellow]" : "[white]";

            table.AddRow($"{metricColor}Execution Time[/]", $"{valueColor}{result.ExecutionTime} ms[/]");
            table.AddRow($"{metricColor}Instructions Executed[/]", $"{valueColor}{result.InstructionsExecuted}[/]");
            table.AddRow($"{metricColor}Instructions Per Second[/]", $"{valueColor}{result.InstructionsPerSecond:F2}[/]");
            table.AddRow($"{metricColor}Output[/]", $"{valueColor}{string.Join(", ", result.Output)}[/]");

            if (isComparison)
                table.BorderColor(Color.Blue);

            AnsiConsole.Write(table);
        }
    }
}

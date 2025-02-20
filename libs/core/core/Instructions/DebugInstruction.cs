using MemoryMachine.Core.Attributes;
using Spectre.Console;

namespace MemoryMachine.Core.Instructions
{
    [RegisterInstruction]
    public class DebugInstruction : IInstruction
    {
        public string Name => "DEBUG";

        public bool Validate(InstructionArgument[] arguments) => arguments.Length == 0;

        public void Execute(MemoryMachine machine, InstructionArgument[] arguments)
        {
            machine.AddBreakpoint(machine.ProgramCounter + 1);

            // Main table
            var table = new Table();
            table.Border(TableBorder.Rounded);
            table.Title("[bold blue]Memory Machine State[/]");

            table.AddColumn("[bold]Property[/]");
            table.AddColumn("[bold]Value[/]");

            table.AddRow(
              new Markup("[bold]Program Counter[/]"),
              new Text(machine.ProgramCounter.ToString())
            );
            table.AddRow(
              new Markup("[bold]Instructions Executed[/]"),
              new Text(machine.ExecutedInstructions.ToString())
            );

            table.AddRow(
              new Markup("[bold]Input Buffer[/]"),
              new Text(string.Join(", ", machine.InputBuffer))
            );
            table.AddRow(
              new Markup("[bold]Output Buffer[/]"),
              new Text(string.Join(", ", machine.OutputBuffer))
            );

            // Separate table for registers
            var registerTable = new Table();
            registerTable.Border(TableBorder.Rounded);
            registerTable.Title("[bold]Registers[/]");

            registerTable.AddColumn("[bold]Register[/]");
            registerTable.AddColumn("[bold]Value[/]");

            for (var i = 0; i < machine.Registers.Count; i++)
            {
                var register = machine.Registers[i];
                string registerValueStr;
                if (register.Value is List<int> list)
                {
                    registerValueStr = string.Join(", ", list);
                }
                else
                {
                    registerValueStr = register.Value?.ToString() ?? "null";
                }

                registerTable.AddRow(new Markup($"[bold]R{i}[/]"), new Text(registerValueStr));
            }

            AnsiConsole.Write(table);
            AnsiConsole.Write(registerTable);

            AnsiConsole.MarkupLine(
              "[yellow]Debugging complete. Press any key to continue execution...[/]"
            );
            Console.ReadKey();
            machine.ContinueExecution();
        }
    }
}

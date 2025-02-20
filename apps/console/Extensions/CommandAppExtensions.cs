using System.CommandLine.Builder;
using MemoryMachine.Console.Commands;

namespace MemoryMachine.Console.Extensions
{
    public static class CommandAppExtensions
    {
        /// <summary>
        /// Registers all  commands with the application's root command.
        /// </summary>
        public static CommandLineBuilder AddCommands(this CommandLineBuilder builder)
        {
            builder.Command.Add(new RunCommand());
            return builder;
        }
    }
}

using MemoryMachine.Core.Attributes;

namespace MemoryMachine.Core.Instructions
{
    [RegisterInstruction]
    public class HaltInstruction : IInstruction
    {
        public string Name => "HALT";

        public bool Validate(InstructionArgument[] arguments)
        {
            return arguments.Length == 0;
        }

        public void Execute(MemoryMachine machine, InstructionArgument[] arguments)
        {
            if (arguments.Length != 0)
            {
                throw new ArgumentException("HALT instruction does not require any arguments.");
            }

            // Stop the execution by setting the IsHalted flag.
            machine.IsHalted = true;
        }
    }
}

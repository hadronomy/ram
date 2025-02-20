using MemoryMachine.Core.Attributes;
using MemoryMachine.Core.Resolvers;

namespace MemoryMachine.Core.Instructions
{
    [RegisterInstruction]
    public class ReadInstruction : IInstruction
    {
        public string Name => "READ";

        public bool Validate(InstructionArgument[] arguments)
        {
            if (arguments.Length != 1)
                return false;

            if (arguments[0].Type != OperandType.Direct)
                return false;

            return true;
        }

        public void Execute(MemoryMachine machine, InstructionArgument[] arguments)
        {
            if (arguments.Length != 1)
            {
                throw new ArgumentException("READ instruction requires one argument.");
            }

            var argument = arguments[0];
            if (argument.Type != OperandType.Direct)
            {
                throw new ArgumentException("READ instruction requires a direct operand (register).");
            }

            var registerIndex = argument.Value;
            var inputValue = machine.ReadInput();

            machine.SetRegister(registerIndex, inputValue);
        }
    }
}

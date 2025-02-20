using MemoryMachine.Core.Attributes;

namespace MemoryMachine.Core.Instructions
{
    [RegisterInstruction]
    public class JumpInstruction : IInstruction
    {
        public string Name => "JUMP";

        public bool Validate(InstructionArgument[] arguments)
        {
            if (arguments.Length != 1)
            {
                return false;
            }

            if (arguments[0].Type != OperandType.Immediate)
            {
                return false;
            }

            return true;
        }

        public void Execute(MemoryMachine machine, InstructionArgument[] arguments)
        {
            if (arguments.Length != 1)
            {
                throw new ArgumentException("JUMP instruction requires one argument: label.");
            }

            var labelArgument = arguments[0];

            // The label address is stored as an immediate value
            var jumpAddress = labelArgument.Value;

            if (jumpAddress < 0 || jumpAddress >= machine.Assembly!.Count)
            {
                throw new InvalidOperationException("Invalid jump address." + jumpAddress);
            }

            machine.ProgramCounter = jumpAddress - 1;
        }
    }
}

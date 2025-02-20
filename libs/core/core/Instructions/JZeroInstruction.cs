using MemoryMachine.Core.Attributes;

namespace MemoryMachine.Core.Instructions
{
    [RegisterInstruction]
    public class JZeroInstruction : IInstruction
    {
        public string Name => "JZERO";

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
                throw new ArgumentException("JZERO instruction requires one argument: label.");
            }

            var labelArgument = arguments[0];

            // The label address is stored as an immediate value
            var jumpAddress = labelArgument.Value;

            if (jumpAddress < 0 || jumpAddress >= machine.Assembly!.Count)
            {
                throw new InvalidOperationException($"Error at instruction {machine.ProgramCounter}: Instruction: {Name}, Arguments: {arguments[0]}, Message: Invalid jump address: {jumpAddress}");
            }

            if (machine.GetRegister(0) == 0)
            {
                machine.ProgramCounter = jumpAddress;
                machine.ProgramCounter--;
            }
        }
    }
}

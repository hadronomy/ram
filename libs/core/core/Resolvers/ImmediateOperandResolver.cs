using MemoryMachine.Core.Instructions;

namespace MemoryMachine.Core.Resolvers
{
    public class ImmediateOperandResolver : IOperandResolver
    {
        public int Resolve(MemoryMachine machine, InstructionArgument argument) => argument.Value;

        public void SetValue(MemoryMachine machine, InstructionArgument argument, int value) =>
            throw new InvalidOperationException("Cannot set value for an immediate operand.");
    }
}

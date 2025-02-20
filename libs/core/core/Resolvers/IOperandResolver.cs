using MemoryMachine.Core.Instructions;

namespace MemoryMachine.Core.Resolvers
{
    public interface IOperandResolver
    {
        int Resolve(MemoryMachine machine, InstructionArgument argument);
        void SetValue(MemoryMachine machine, InstructionArgument argument, int value);
    }
}

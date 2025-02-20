using MemoryMachine.Core.Instructions;

namespace MemoryMachine.Core.Resolvers
{
    public class IndirectOperandResolver : IOperandResolver
    {
        public int Resolve(MemoryMachine machine, InstructionArgument argument)
        {
            var indirectAddress = machine.GetRegister(argument.Value);
            return machine.GetRegister(indirectAddress);
        }

        public void SetValue(MemoryMachine machine, InstructionArgument argument, int value)
        {
            var indirectAddress = machine.GetRegister(argument.Value);
            machine.SetRegister(indirectAddress, value);
        }
    }
}

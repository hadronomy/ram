using MemoryMachine.Core.Attributes;
using MemoryMachine.Core.Resolvers;

namespace MemoryMachine.Core.Instructions
{
    [RegisterInstruction]
    public class LoadInstruction : IInstruction
    {
        public string Name => "LOAD";

        public bool Validate(InstructionArgument[] arguments)
        {
            if (arguments.Length != 1)
            {
                return false;
            }

            if (arguments[0].Type != OperandType.Direct &&
                arguments[0].Type != OperandType.Indirect &&
                arguments[0].Type != OperandType.Immediate)
            {
                return false;
            }

            return true;
        }

        public void Execute(MemoryMachine machine, InstructionArgument[] arguments)
        {
            if (arguments.Length != 1)
            {
                throw new ArgumentException("LOAD instruction requires exactly one argument.");
            }
            var sourceArgument = arguments[0];
            var sourceResolver = OperandResolverFactory.Instance.GetResolver(
                sourceArgument.Type
            );
            var sourceValue = sourceResolver.Resolve(machine, sourceArgument);

            machine.SetRegister(0, sourceValue);
        }
    }
}

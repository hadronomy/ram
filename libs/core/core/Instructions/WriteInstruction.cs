using MemoryMachine.Core.Attributes;
using MemoryMachine.Core.Resolvers;

namespace MemoryMachine.Core.Instructions
{
    [RegisterInstruction]
    public class WriteInstruction : IInstruction
    {
        public string Name => "WRITE";

        public bool Validate(InstructionArgument[] arguments)
        {
            if (arguments.Length != 1)
            {
                return false;
            }

            if (arguments[0].Type != OperandType.Direct &&
                arguments[0].Type != OperandType.Immediate &&
                arguments[0].Type != OperandType.Indirect)
            {
                return false;
            }

            return true;
        }

        public void Execute(MemoryMachine machine, InstructionArgument[] arguments)
        {
            if (arguments.Length != 1)
            {
                throw new ArgumentException("WRITE instruction requires one argument.");
            }

            var sourceArgument = arguments[0];

            var sourceResolver = OperandResolverFactory.Instance.GetResolver(
                sourceArgument.Type
            );

            var valueToWrite = sourceResolver.Resolve(machine, sourceArgument);

            machine.WriteOutput(valueToWrite);
        }
    }
}

using MemoryMachine.Core.Attributes;
using MemoryMachine.Core.Resolvers;

namespace MemoryMachine.Core.Instructions
{
    [RegisterInstruction]
    public class StoreInstruction : IInstruction
    {
        public string Name => "STORE";

        public bool Validate(InstructionArgument[] arguments)
        {
            if (arguments.Length != 1)
            {
                return false;
            }

            if (arguments[0].Type != OperandType.Direct &&
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
                throw new ArgumentException(
                    "STORE instruction requires one argument: destination."
                );
            }

            var destinationArgument = arguments[0];

            var destinationResolver = OperandResolverFactory.Instance.GetResolver(
                destinationArgument.Type
            );

            // Source is always R0
            var sourceValue = machine.GetRegister(0);

            destinationResolver.SetValue(machine, destinationArgument, sourceValue);
        }
    }
}

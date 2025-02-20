using MemoryMachine.Core.Attributes;
using MemoryMachine.Core.Resolvers;

namespace MemoryMachine.Core.Instructions
{
    [RegisterInstruction]
    public class AddInstruction : IInstruction
    {
        public string Name => "ADD";

        public bool Validate(InstructionArgument[] arguments)
        {
            if (arguments.Length != 1)
            {
                return false;
            }

            if (arguments[0].Type != OperandType.Direct && arguments[0].Type != OperandType.Immediate)
            {
                return false;
            }

            return true;
        }

        public void Execute(MemoryMachine machine, InstructionArgument[] arguments)
        {
            var argument = arguments[0];
            var resolver = OperandResolverFactory.Instance.GetResolver(argument.Type);
            var valueToAdd = resolver.Resolve(machine, argument);

            var currentR0Value = machine.GetRegister(0);
            var result = currentR0Value + valueToAdd;

            machine.SetRegister(0, result);
        }
    }
}

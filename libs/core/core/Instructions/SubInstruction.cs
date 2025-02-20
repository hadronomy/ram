using MemoryMachine.Core.Attributes;
using MemoryMachine.Core.Resolvers;

namespace MemoryMachine.Core.Instructions
{
    [RegisterInstruction]
    public class SubInstruction : IInstruction
    {
        public string Name => "SUB";

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
                throw new ArgumentException(
                    "SUB instruction requires one argument: operand to subtract from R0."
                );
            }

            var operandArgument = arguments[0];

            var operandResolver = OperandResolverFactory.Instance.GetResolver(
                operandArgument.Type
            );

            var operandValue = operandResolver.Resolve(machine, operandArgument);

            var currentR0Value = machine.GetRegister(0);
            var result = currentR0Value - operandValue;

            machine.SetRegister(0, result);
        }
    }
}

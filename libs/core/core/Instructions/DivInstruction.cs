using MemoryMachine.Core.Attributes;
using MemoryMachine.Core.Resolvers;

namespace MemoryMachine.Core.Instructions
{
    [RegisterInstruction]
    public class DivInstruction : IInstruction
    {
        public string Name => "DIV";

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
                    "DIV instruction requires one argument: operand to divide R0 by."
                );
            }

            var operandArgument = arguments[0];

            var operandResolver = OperandResolverFactory.Instance.GetResolver(
                operandArgument.Type
            );

            var operandValue = operandResolver.Resolve(machine, operandArgument);

            if (operandValue == 0)
            {
                throw new DivideByZeroException("Cannot divide R0 by zero.");
            }

            var currentR0Value = machine.GetRegister(0);
            var result = currentR0Value / operandValue;

            machine.SetRegister(0, result);
        }
    }
}

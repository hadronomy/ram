using MemoryMachine.Core.Instructions;
using MemoryMachine.Core.Parser;
using System.Linq;

namespace MemoryMachine.Core.Visitors
{
    public class InstructionVisitor : MemoryMachineAssemblyBaseVisitor<
        (IInstruction Instruction, InstructionArgument[] Arguments)
    >
    {
        private readonly InstructionSet _instructionSet;
        private readonly ArgumentVisitor _operandVisitor = new();
        private readonly Dictionary<string, int> _labelAddresses;

        public InstructionVisitor(
            InstructionSet instructionSet,
            Dictionary<string, int> labelAddresses)
        {
            _instructionSet = instructionSet;
            _labelAddresses = labelAddresses;
        }

        public override (IInstruction Instruction, InstructionArgument[] Arguments)
        VisitInstruction(MemoryMachineAssemblyParser.InstructionContext context)
        {
            var instructionName = context.IDENTIFIER().GetText().ToUpper();

            if (!_instructionSet.HasInstruction(instructionName))
            {
                throw new InvalidOperationException(
                    $"Unknown instruction: {instructionName}"
                );
            }

            var instruction = _instructionSet.GetInstruction(instructionName);
            List<InstructionArgument> arguments = new();

            if (context.argument() != null)
            {
                var argument = _operandVisitor.Visit(context.argument());

                // Resolve label if present
                if (argument.Label != null)
                {
                    var labelName = argument.Label;
                    if (_labelAddresses.ContainsKey(labelName))
                    {
                        argument = new InstructionArgument(
                            OperandType.Immediate,
                            _labelAddresses[labelName],
                            argument.Index
                        );
                    }
                    else
                    {
                        throw new InvalidOperationException($"Label not found: {argument.Label}");
                    }
                }

                arguments.Add(argument);
            }

            return (instruction, arguments.ToArray());
        }

        public (IInstruction Instruction, InstructionArgument[] Arguments)
        VisitLabelDefinition(MemoryMachineAssemblyParser.Label_definitionContext context)
        {
            return VisitInstruction(context.instruction());
        }
    }

}

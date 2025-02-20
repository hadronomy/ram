
using Antlr4.Runtime.Misc;
using MemoryMachine.Core.Instructions;
using MemoryMachine.Core.Parser;

namespace MemoryMachine.Core.Visitors
{
    public class ArgumentVisitor : MemoryMachineAssemblyBaseVisitor<InstructionArgument>
    {
        public override InstructionArgument VisitNumberOperand(
        MemoryMachineAssemblyParser.NumberOperandContext context
    )
        {
            return VisitDirect(context.direct());
        }

        public override InstructionArgument VisitIndirectOperand(
            MemoryMachineAssemblyParser.IndirectOperandContext context
        )
        {
            return VisitIndirect(context.indirect());
        }

        public override InstructionArgument VisitImmediateOperand(
            MemoryMachineAssemblyParser.ImmediateOperandContext context
        )
        {
            return VisitImmediate(context.immediate());
        }

        public override InstructionArgument VisitLabelOperand(
            MemoryMachineAssemblyParser.LabelOperandContext context
        )
        {
            // Labels are resolved later, just store the name for now
            string labelName = context.label().IDENTIFIER().GetText();
            return new InstructionArgument(OperandType.Immediate, 0)
            {
                Label = labelName
            };
        }
        public override InstructionArgument VisitDirect(MemoryMachineAssemblyParser.DirectContext context)
        {
            var value = int.Parse(context.NUMBER().GetText());
            InstructionArgument? indexArgument = null;

            if (context.accessor() != null)
            {
                var indexContext = context.accessor().index();
                indexArgument = Visit(indexContext);
            }

            return new InstructionArgument(OperandType.Direct, value, indexArgument);
        }

        public override InstructionArgument VisitIndirect(MemoryMachineAssemblyParser.IndirectContext context)
        {
            var value = int.Parse(context.NUMBER().GetText());
            return new InstructionArgument(OperandType.Indirect, value);
        }

        public override InstructionArgument VisitImmediate(MemoryMachineAssemblyParser.ImmediateContext context)
        {
            int value = int.Parse(context.NUMBER().GetText());
            return new InstructionArgument(OperandType.Immediate, value);
        }

        public override InstructionArgument VisitIndex([NotNull] MemoryMachineAssemblyParser.IndexContext context)
        {
            if (context.direct() != null)
            {
                return VisitDirect(context.direct());
            }
            else if (context.indirect() != null)
            {
                return VisitIndirect(context.indirect());
            }
            else if (context.immediate() != null)
            {
                return VisitImmediate(context.immediate());
            }
            else
            {
                throw new InvalidOperationException("Invalid index type");
            }
        }
    }

}

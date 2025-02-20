using MemoryMachine.Core.Parser;

namespace MemoryMachine.Core.Visitors
{
    public class LabelVisitor : MemoryMachineAssemblyBaseVisitor<object?>
    {
        private readonly Dictionary<string, int> _labelAddresses = new();
        private int _currentAddress = 0;

        public Dictionary<string, int> LabelAddresses => _labelAddresses;

        public override object? VisitLine(MemoryMachineAssemblyParser.LineContext context)
        {
            if (context.label_definition() != null)
            {
                VisitLabel_definition(context.label_definition());
                _currentAddress++;
            }
            else if (context.instruction() != null)
            {
                _currentAddress++;
            }
            return null;
        }

        public override object? VisitLabel_definition(
            MemoryMachineAssemblyParser.Label_definitionContext context
        )
        {
            string labelName = context.IDENTIFIER().GetText();
            _labelAddresses[labelName] = _currentAddress;
            return null;
        }
    }
}

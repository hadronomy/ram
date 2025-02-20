using System.Collections;
using MemoryMachine.Core.Instructions;

namespace MemoryMachine.Core
{
    public class Assembly : IEnumerable<(IInstruction Instruction, InstructionArgument[] Arguments)>
    {
        private readonly List<(IInstruction Instruction, InstructionArgument[] Arguments)> _instructions;
        private readonly Dictionary<string, int> _labelAddresses;

        public IReadOnlyList<(IInstruction Instruction, InstructionArgument[] Arguments)> Instructions
            => _instructions.AsReadOnly();

        public IReadOnlyDictionary<string, int> LabelAddresses
            => _labelAddresses;

        public int Count => _instructions.Count;

        public Assembly(
            List<(IInstruction Instruction, InstructionArgument[] Arguments)> instructions,
            Dictionary<string, int> labelAddresses)
        {
            _instructions = instructions;
            _labelAddresses = labelAddresses;
        }

        public (IInstruction Instruction, InstructionArgument[] Arguments) this[int index]
            => _instructions[index];

        public int GetLabelAddress(string label)
            => _labelAddresses.TryGetValue(label, out var address) ? address : -1;

        public bool HasLabel(string label)
            => _labelAddresses.ContainsKey(label);

        public IEnumerator<(IInstruction Instruction, InstructionArgument[] Arguments)> GetEnumerator()
            => _instructions.GetEnumerator();

        IEnumerator IEnumerable.GetEnumerator()
            => GetEnumerator();
    }
}

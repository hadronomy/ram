namespace MemoryMachine.Core.Instructions
{
    public class InstructionArgument
    {
        public OperandType Type { get; }
        public int Value { get; }
        public InstructionArgument? Index { get; }
        public string? Label { get; internal set; }

        public InstructionArgument(OperandType type, int value, InstructionArgument? index = null, string? label = null)
        {
            Type = type;
            Value = value;
            Index = index;
            Label = label;
        }

        public override string ToString() =>
            $"{Type}: {Value}{(Index != null ? $"[{Index.Value}]" : "")}{(string.IsNullOrEmpty(Label) ? "" : $" ({Label})")}";
    }
}

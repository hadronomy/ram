namespace MemoryMachine.Core.Instructions
{
    public interface IInstruction
    {
        void Execute(MemoryMachine machine, InstructionArgument[] arguments);
        bool Validate(InstructionArgument[] arguments);

        string Name { get; }
    }
}

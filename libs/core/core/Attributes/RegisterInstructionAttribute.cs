using MemoryMachine.Core.Instructions;

namespace MemoryMachine.Core.Attributes
{
    [AttributeUsage(AttributeTargets.Class)]
    public sealed class RegisterInstructionAttribute : Attribute
    {
        public RegisterInstructionAttribute()
        {
        }
    }
}

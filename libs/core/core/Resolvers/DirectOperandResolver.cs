using MemoryMachine.Core.Instructions;

namespace MemoryMachine.Core.Resolvers
{
    public class DirectOperandResolver : IOperandResolver
    {
        public int Resolve(MemoryMachine machine, InstructionArgument argument)
        {
            if (argument.Index == null)
            {
                return machine.GetRegister(argument.Value);
            }

            var indexResolver = OperandResolverFactory.Instance.GetResolver(argument.Index.Type);
            var indexValue = indexResolver.Resolve(machine, argument.Index);

            return machine.Registers[argument.Value].Get<int>(indexValue);
        }


        public void SetValue(MemoryMachine machine, InstructionArgument argument, int value)
        {
            if (argument.Index == null)
            {
                machine.SetRegister(argument.Value, value);
                return;
            }

            var indexResolver = OperandResolverFactory.Instance.GetResolver(argument.Index.Type);
            var indexValue = indexResolver.Resolve(machine, argument.Index);

            var register = machine.Registers[argument.Value];
            if (register.Value is not List<int>)
            {
                machine.SetRegister(argument.Value, new List<int>());
            }
            machine.Registers[argument.Value].Set(value, indexValue);
        }

    }
}

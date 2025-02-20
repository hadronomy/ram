using System.Collections.Generic;

namespace MemoryMachine.Core.Registers
{
    public class Register
    {
        public static readonly Register Zero = new Register(0);
        public static readonly Register One = new Register(1);

        private readonly IRegisterType _type;

        public object Value => _type.Value;

        public Register()
        {
            _type = new ScalarRegisterType(0);
        }

        public Register(int value)
        {
            _type = new ScalarRegisterType(value);
        }

        public Register(List<int> vector)
        {
            _type = new VectorRegisterType(vector);
        }

        public static implicit operator int(Register register)
        {
            return register._type.Get<int>();
        }

        public static implicit operator Register(int value) => new Register(value);

        public T Get<T>(int index = 0) => _type.Get<T>(index);
        public void Set<T>(T value, int index = 0) => _type.Set(value, index);
    }
}

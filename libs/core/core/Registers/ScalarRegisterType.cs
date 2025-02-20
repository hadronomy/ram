namespace MemoryMachine.Core.Registers
{
    public class ScalarRegisterType : IRegisterType
    {
        private int _value;

        public ScalarRegisterType(int value)
        {
            _value = value;
        }

        public object Value => _value;

        public T Get<T>(int index = 0)
        {
            if (typeof(T) != typeof(int))
                throw new InvalidOperationException($"Cannot convert scalar register to {typeof(T)}");

            return (T)(object)_value;
        }

        public void Set<T>(T value, int index = 0)
        {
            if (value is int intValue)
                _value = intValue;
            else
                throw new InvalidOperationException($"Cannot set scalar register to {typeof(T)}");
        }
    }
}

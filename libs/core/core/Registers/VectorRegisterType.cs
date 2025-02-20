namespace MemoryMachine.Core
{
    public class VectorRegisterType : IRegisterType
    {
        private List<int> _vector;

        public VectorRegisterType(List<int> vector)
        {
            _vector = vector;
        }

        public object Value => _vector;

        public T Get<T>(int index = 0)
        {
            if (index < 0 || index >= _vector.Count)
                throw new ArgumentOutOfRangeException(nameof(index));

            if (typeof(T) == typeof(int))
                return (T)(object)_vector[index];
            else if (typeof(T) == typeof(List<int>))
                return (T)(object)_vector;

            throw new InvalidOperationException($"Cannot convert vector register to {typeof(T)}");
        }

        public void Set<T>(T value, int index = 0)
        {
            if (value is int intValue)
            {
                if (index < 0 || index >= _vector.Count)
                {
                    int requiredSize = index + 1;
                    while (_vector.Count < requiredSize)
                    {
                        _vector.Add(0);
                    }
                }
                _vector[index] = intValue;
            }
            else if (value is List<int> vectorValue)
            {
                _vector = vectorValue;
            }
            else
                throw new InvalidOperationException($"Cannot set vector register to {typeof(T)}");
        }
    }
}

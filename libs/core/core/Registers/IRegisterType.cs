namespace MemoryMachine.Core
{
    public interface IRegisterType
    {
        object Value { get; }
        T Get<T>(int index = 0);
        void Set<T>(T value, int index = 0);
    }
}

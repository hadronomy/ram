namespace MemoryMachine.Console.Handlers
{
    public interface ICommandHandler<in TOptions>
    {
        Task ExecuteAsync(TOptions options);
    }
}

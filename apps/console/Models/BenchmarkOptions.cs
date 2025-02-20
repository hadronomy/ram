namespace MemoryMachine.Console.Models
{
    public record BenchmarkOptions(
        int[] Sizes,
        int Iterations,
        int Warmup,
        string[] Strategies
    );
}

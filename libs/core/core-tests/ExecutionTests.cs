using MemoryMachine.Core;
using MemoryMachine.Core.Instructions;

namespace MemoryMachine.Tests
{
    using MemoryMachine = Core.MemoryMachine;

    public class ExecutionTests
    {
        [Fact]
        public void Program1_CopiesInputToOutputUntilZero()
        {
            var machine = new MemoryMachine();
            var instructionSet = new InstructionSet();
            var assemblyLoader = new AssemblyLoader(instructionSet);
            var assembly = assemblyLoader.LoadFromString(Programs.Program1);
            machine.LoadAssembly(assembly);

            machine.QueueInputRange([1, 2, 3, 0, 4, 5]);
            machine.Execute();

            machine.OutputBuffer.ShouldBe([1, 2, 3]);
        }

        [Fact]
        public void Program2_RecognizesLanguageWithEqualOnesAndTwos()
        {
            var machine = new MemoryMachine();
            var instructionSet = new InstructionSet();
            var assemblyLoader = new AssemblyLoader(instructionSet);
            var assembly = assemblyLoader.LoadFromString(Programs.Program2);
            machine.LoadAssembly(assembly);

            // Test case 1: Valid word (11220)
            machine.QueueInputRange([1, 1, 2, 2, 0]);
            machine.Execute();
            machine.OutputBuffer.ShouldBe([1]);

            machine.Reset();
            machine.QueueInputRange([1, 2, 1, 2, 0]);
            machine.Execute();
            machine.OutputBuffer.ShouldBe([1]);

            machine.Reset();
            machine.QueueInputRange([1, 2, 2, 1, 0]);
            machine.Execute();
            machine.OutputBuffer.ShouldBe([1]);

            machine.Reset();
            machine.QueueInputRange([2, 1, 1, 2, 0]);
            machine.Execute();
            machine.OutputBuffer.ShouldBe([1]);

            machine.Reset();
            machine.QueueInputRange([2, 1, 2, 1, 0]);
            machine.Execute();
            machine.OutputBuffer.ShouldBe([1]);

            machine.Reset();
            machine.QueueInputRange([2, 2, 1, 1, 0]);
            machine.Execute();
            machine.OutputBuffer.ShouldBe([1]);

            machine.Reset();
            // Test case 2: Invalid word (12220)
            machine.Reset();
            machine.QueueInputRange([1, 2, 2, 2, 0]);
            machine.Execute();
            machine.OutputBuffer.ShouldBe([0]);

            machine.Reset();
            machine.QueueInputRange([1, 1, 1, 2, 0]);
            machine.Execute();
            machine.OutputBuffer.ShouldBe([0]);

            machine.Reset();
            machine.QueueInputRange([1, 1, 2, 2, 2, 0]);
            machine.Execute();
            machine.OutputBuffer.ShouldBe([0]);
        }

        [Fact]
        public void Program4_DoublesInputToOutputUntilZero()
        {
            var machine = new MemoryMachine();
            var instructionSet = new InstructionSet();
            var assemblyLoader = new AssemblyLoader(instructionSet);
            var assembly = assemblyLoader.LoadFromString(Programs.Program4);
            machine.LoadAssembly(assembly);

            machine.QueueInputRange([1, 2, 3, 0, 4, 5]);
            machine.Execute();

            machine.OutputBuffer.ShouldBe([2, 4, 6]);
        }

        [Fact]
        public void Program5_SumsInputToOutputUntilZero()
        {
            var machine = new MemoryMachine();
            var instructionSet = new InstructionSet();
            var assemblyLoader = new AssemblyLoader(instructionSet);
            var assembly = assemblyLoader.LoadFromString(Programs.Program5);
            machine.LoadAssembly(assembly);

            machine.QueueInputRange([1, 2, 3, 0, 4, 5]);
            machine.Execute();

            machine.OutputBuffer.ShouldBe([6]);
        }

        [Fact]
        public void Program7_ReadsToRegistersMultipliesBy3AndWrites()
        {
            var machine = new MemoryMachine();
            var instructionSet = new InstructionSet();
            var assemblyLoader = new AssemblyLoader(instructionSet);
            var assembly = assemblyLoader.LoadFromString(Programs.Program7);
            machine.LoadAssembly(assembly);

            machine.QueueInputRange([1, 2, 3, 0, 4, 5]);
            machine.Execute();

            machine.OutputBuffer.ShouldBe([3, 6, 9]);
        }

        [Fact]
        public void ArrayToRegisters()
        {
            // Arrange
            var machine = new MemoryMachine();
            var instructionSet = new InstructionSet();
            var assemblyLoader = new AssemblyLoader(instructionSet);
            var assembly = assemblyLoader.LoadFromString(Programs.ArrayToRegisters);
            machine.LoadAssembly(assembly);

            // Initialize registers and array
            machine.QueueInputRange([5, 2, 9, 1, 5, 6]);
            machine.Execute();

            // Verify the sorted array
            List<int> expectedArray = [2, 9, 1, 5, 6];

            machine.OutputBuffer.ShouldBe(expectedArray);

        }

        [Fact]
        public void ModOperatorTest()
        {
            // Arrange
            var machine = new MemoryMachine();
            var instructionSet = new InstructionSet();
            var assemblyLoader = new AssemblyLoader(instructionSet);
            var assembly = assemblyLoader.LoadFromString(Programs.ModOperator);
            machine.LoadAssembly(assembly);

            // Initialize input
            machine.QueueInputRange([10, 3]);
            machine.Execute();

            // Verify the output
            List<int> expectedOutput = [1];
            machine.OutputBuffer.ShouldBe(expectedOutput);
        }
    }
}

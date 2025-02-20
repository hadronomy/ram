using System.Reflection;
using MemoryMachine.Core.Attributes;

namespace MemoryMachine.Core.Instructions
{
    public class InstructionSet
    {
        private readonly Dictionary<string, IInstruction> _instructions;

        public IReadOnlyDictionary<string, IInstruction> Instructions => _instructions;

        public InstructionSet()
        {
            _instructions = new Dictionary<string, IInstruction>(StringComparer.OrdinalIgnoreCase);
            RegisterDefaultInstructions();
        }

        private void RegisterDefaultInstructions()
        {
            // Get all types in the current app domain that implement IInstruction
            var instructionTypes = AppDomain.CurrentDomain.GetAssemblies()
                .SelectMany(assembly => assembly.GetTypes())
                .Where(type =>
                    type.GetCustomAttribute<RegisterInstructionAttribute>() != null &&
                    !type.IsAbstract &&
                    typeof(IInstruction).IsAssignableFrom(type));

            foreach (var type in instructionTypes)
            {
                try
                {
                    if (Activator.CreateInstance(type) is IInstruction instruction)
                    {
                        RegisterInstruction(instruction);
                    }
                }
                catch (Exception ex)
                {
                    throw new InvalidOperationException(
                        $"Failed to create instruction: {type.Name}", ex);
                }
            }
        }

        public void RegisterInstruction(IInstruction instruction)
        {
            if (instruction == null)
                throw new ArgumentNullException(nameof(instruction));

            if (string.IsNullOrWhiteSpace(instruction.Name))
                throw new ArgumentException("Instruction name cannot be empty", nameof(instruction));

            _instructions[instruction.Name] = instruction;
        }

        public IInstruction GetInstruction(string name)
        {
            if (string.IsNullOrWhiteSpace(name))
                throw new ArgumentException("Instruction name cannot be empty", nameof(name));

            return _instructions.TryGetValue(name, out var instruction)
                ? instruction
                : throw new KeyNotFoundException($"Instruction '{name}' not found");
        }

        public bool HasInstruction(string name)
            => !string.IsNullOrWhiteSpace(name) && _instructions.ContainsKey(name);

        public IEnumerable<string> GetAvailableInstructions()
            => _instructions.Keys.OrderBy(k => k);
    }
}

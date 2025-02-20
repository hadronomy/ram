using MemoryMachine.Core.Registers;
using System.Collections.Generic;
using System.Linq;
using System;
using System.Threading;

namespace MemoryMachine.Core
{
    public class MemoryMachine
    {
        private const int RegisterCount = 32;
        private readonly List<Register> _registers;
        private readonly Queue<int> _inputBuffer;
        private readonly List<int> _outputBuffer;
        private Assembly? _assembly;
        private readonly HashSet<int> _breakpoints;
        private readonly AutoResetEvent _continueExecutionEvent = new(false);

        public int ProgramCounter { get; set; }
        public IReadOnlyList<Register> Registers => _registers.AsReadOnly();
        public IReadOnlyCollection<int> InputBuffer => _inputBuffer;
        public IReadOnlyList<int> OutputBuffer => _outputBuffer.AsReadOnly();
        public Assembly? Assembly => _assembly;

        public bool IsHalted { get; set; }
        public int ExecutedInstructions { get; private set; }

        public MemoryMachine()
        {
            _registers = new List<Register>(RegisterCount);
            for (var i = 0; i < RegisterCount; i++)
            {
                _registers.Add(Register.Zero);
            }
            _inputBuffer = new Queue<int>();
            _outputBuffer = [];
            _breakpoints = [];
            ProgramCounter = 0;
            IsHalted = false;
            ExecutedInstructions = 0;
        }

        public void LoadAssembly(Assembly assembly)
        {
            Reset();
            _assembly = assembly;
        }

        public void Reset()
        {
            ProgramCounter = 0;
            _inputBuffer.Clear();
            _outputBuffer.Clear();
            for (var i = 0; i < _registers.Count; i++)
            {
                _registers[i] = Register.Zero;
            }
            IsHalted = false;
            ExecutedInstructions = 0;
        }

        public void SetRegister(int register, object value)
        {
            if (register >= _registers.Count)
            {
                while (_registers.Count <= register)
                {
                    _registers.Add(new Register());
                }
            }

            if (value is List<int> vectorValue)
            {
                _registers[register] = new Register(vectorValue);
            }
            else if (value is int intValue)
            {
                _registers[register] = new Register(intValue);
            }
            else
            {
                throw new ArgumentException($"Unsupported register value type: {value?.GetType()}");
            }
        }

        public int GetRegister(int register)
            => register < _registers.Count ? _registers[register] : 0;

        public void QueueInput(int value)
            => _inputBuffer.Enqueue(value);

        public void QueueInputRange(IEnumerable<int> values)
        {
            foreach (var value in values)
            {
                _inputBuffer.Enqueue(value);
            }
        }

        public int ReadInput()
            => _inputBuffer.Count > 0 ? _inputBuffer.Dequeue() : 0;

        public void WriteOutput(int value)
            => _outputBuffer.Add(value);

        public void AddBreakpoint(int address)
        {
            _breakpoints.Add(address);
        }

        public void RemoveBreakpoint(int address)
        {
            _breakpoints.Remove(address);
        }

        public void ClearBreakpoints()
        {
            _breakpoints.Clear();
        }

        public bool IsBreakpointSet(int address)
        {
            return _breakpoints.Contains(address);
        }

        public void ContinueExecution()
        {
            _continueExecutionEvent.Set();
        }

        public void Execute()
        {
            if (_assembly == null)
                throw new InvalidOperationException("No assembly loaded");

            while (ProgramCounter < _assembly.Count && !IsHalted)
            {
                if (_breakpoints.Contains(ProgramCounter))
                {
                    _continueExecutionEvent.WaitOne();
                }

                var (instruction, arguments) = _assembly[ProgramCounter];
                try
                {
                    ExecutedInstructions++;
                    instruction.Execute(this, arguments);
                }
                catch (Exception e)
                {
                    var registerDump = string.Join("\n", _registers.Select((r, i) =>
                    {
                        var valueString = r.Value is List<int> list ? list.FirstOrDefault().ToString() : r.Value?.ToString() ?? "null";
                        return $"R{i}: {valueString}";
                    }));
                    throw new InvalidOperationException($"Error at instruction {ProgramCounter}: Instruction: {instruction.GetType().Name}, Arguments: {string.Join(", ", arguments.Select(a => a.ToString()))}, Message: {e.Message}\nRegister Dump:\n{registerDump}", e);
                }
                ProgramCounter++;
            }
        }
    }
}

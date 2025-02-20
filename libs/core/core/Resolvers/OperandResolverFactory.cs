

namespace MemoryMachine.Core.Resolvers
{
    public class OperandResolverFactory
    {
        private static OperandResolverFactory? _instance;
        private static readonly object _lock = new();

        private readonly Dictionary<OperandType, IOperandResolver> _resolvers = new();

        private OperandResolverFactory()
        {
            _resolvers[OperandType.Immediate] = new ImmediateOperandResolver();
            _resolvers[OperandType.Direct] = new DirectOperandResolver();
            _resolvers[OperandType.Indirect] = new IndirectOperandResolver();
        }

        public static OperandResolverFactory Instance
        {
            get
            {
                if (_instance == null)
                {
                    lock (_lock)
                    {
                        if (_instance == null)
                        {
                            _instance = new OperandResolverFactory();
                        }
                    }
                }
                return _instance;
            }
        }

        public IOperandResolver GetResolver(OperandType type)
        {
            if (!_resolvers.TryGetValue(type, out var resolver))
            {
                throw new ArgumentException($"Unsupported operand type: {type}");
            }
            return resolver;
        }
    }
}

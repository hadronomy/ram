using Antlr4.Runtime;
using MemoryMachine.Core.Instructions;
using MemoryMachine.Core.Parser;
using MemoryMachine.Core.Visitors;
using System.Text;

namespace MemoryMachine.Core
{
    public class AssemblyLoadException : Exception
    {
        public AssemblyLoadException(string message) : base(message) { }
        public AssemblyLoadException(string message, Exception innerException) : base(message, innerException) { }
    }

    public class AssemblyLoader(InstructionSet instructionSet)
    {
        private readonly InstructionSet _instructionSet = instructionSet;

        public Assembly LoadFromString(string sourceCode)
        {
            // Create the lexer and parser
            var inputStream = new AntlrInputStream(sourceCode);
            var lexer = new MemoryMachineAssemblyLexer(inputStream);
            var tokenStream = new CommonTokenStream(lexer);
            var parser = new MemoryMachineAssemblyParser(tokenStream);

            var errorListener = new AssemblyErrorListener();
            parser.RemoveErrorListeners();
            parser.AddErrorListener(errorListener);

            // Get the parse tree
            var tree = parser.program();

            // First pass: collect all labels and their addresses
            var labelVisitor = new LabelVisitor();
            labelVisitor.Visit(tree);

            // Second pass: parse instructions with resolved labels
            var instructionVisitor = new InstructionVisitor(_instructionSet, labelVisitor.LabelAddresses);
            var instructions = new List<(IInstruction, InstructionArgument[])>();

            // Visit each line and collect instructions
            foreach (var line in tree.line())
            {
                if (line.instruction() != null)
                {
                    var instruction = instructionVisitor.VisitInstruction(line.instruction());
                    instructions.Add(instruction);
                }
                else if (line.label_definition() != null)
                {
                    var instruction = instructionVisitor.VisitLabelDefinition(line.label_definition());
                    instructions.Add(instruction);
                }
            }

            if (errorListener.HasError)
            {
                throw new AssemblyLoadException("Syntax error in assembly code.");
            }

            return new Assembly(instructions, labelVisitor.LabelAddresses);
        }

        public Assembly LoadFromFile(string filePath)
        {
            var sourceCode = File.ReadAllText(filePath, Encoding.UTF8);
            return LoadFromString(sourceCode);
        }
    }
}

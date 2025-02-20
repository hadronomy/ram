namespace MemoryMachine.Tests;

using Antlr4.Runtime;
using MemoryMachine.Core.Parser;
using Snapshooter.Xunit;

/// <summary>
/// Tests for the memory machine assembly parser
/// </summary>
public class ParserTests
{


    [Fact(DisplayName = "Parse Program1 Should Create Valid AST")]
    public void ParseProgram1ShouldCreateValidAST()
    {
        // Arrange
        string input = Programs.Program1;

        var inputStream = new AntlrInputStream(input);
        var lexer = new MemoryMachineAssemblyLexer(inputStream);
        var tokens = new CommonTokenStream(lexer);
        var parser = new MemoryMachineAssemblyParser(tokens);

        var tree = parser.program();
        var stringRepresentation = tree.ToStringTree(parser);

        stringRepresentation.MatchSnapshot();
    }

    /// <summary>
    /// Parse Program2 Should Create Valid AST
    /// </summary>
    [Fact(DisplayName = "Parse Program2 Should  Create Valid AST")]
    public void ParseProgram2ShouldCreateValidAST()
    {
        // Arrange
        string input = Programs.Program2;

        var inputStream = new AntlrInputStream(input);
        var lexer = new MemoryMachineAssemblyLexer(inputStream);
        var tokens = new CommonTokenStream(lexer);
        var parser = new MemoryMachineAssemblyParser(tokens);

        var tree = parser.program();
        var stringRepresentation = tree.ToStringTree(parser);

        stringRepresentation.MatchSnapshot();
    }

    [Fact(DisplayName = "Parse Program3 Should Create Valid AST")]
    public void ParseProgram3ShouldCreateValidAST()
    {

        string input = Programs.Program3;

        var inputStream = new AntlrInputStream(input);
        var lexer = new MemoryMachineAssemblyLexer(inputStream);
        var tokens = new CommonTokenStream(lexer);
        var parser = new MemoryMachineAssemblyParser(tokens);

        var tree = parser.program();
        var stringRepresentation = tree.ToStringTree(parser);

        stringRepresentation.MatchSnapshot();
    }

    [Fact(DisplayName = "Parse Program4 Should Create Valid AST")]
    public void ParseProgram4ShouldCreateValidAST()
    {
        string input = Programs.Program4;

        var inputStream = new AntlrInputStream(input);
        var lexer = new MemoryMachineAssemblyLexer(inputStream);
        var tokens = new CommonTokenStream(lexer);
        var parser = new MemoryMachineAssemblyParser(tokens);

        var tree = parser.program();
        var stringRepresentation = tree.ToStringTree(parser);

        stringRepresentation.MatchSnapshot();
    }

    [Fact(DisplayName = "Parse Program5 Should Create Valid AST")]
    public void ParseProgram5ShouldCreateValidAST()
    {
        string input = Programs.Program5;

        var inputStream = new AntlrInputStream(input);
        var lexer = new MemoryMachineAssemblyLexer(inputStream);
        var tokens = new CommonTokenStream(lexer);
        var parser = new MemoryMachineAssemblyParser(tokens);

        var tree = parser.program();
        var stringRepresentation = tree.ToStringTree(parser);

        stringRepresentation.MatchSnapshot();
    }

    [Fact(DisplayName = "Parse Program6 Should Create Valid AST")]
    public void ParseProgram6ShouldCreateValidAST()
    {
        string input = Programs.Program6;

        var inputStream = new AntlrInputStream(input);
        var lexer = new MemoryMachineAssemblyLexer(inputStream);
        var tokens = new CommonTokenStream(lexer);
        var parser = new MemoryMachineAssemblyParser(tokens);

        var tree = parser.program();
        var stringRepresentation = tree.ToStringTree(parser);

        stringRepresentation.MatchSnapshot();
    }

    [Fact(DisplayName = "Parse Program7 Should Create Valid AST")]
    public void ParseProgram7ShouldCreateValidAST()
    {
        string input = Programs.Program7;

        var inputStream = new AntlrInputStream(input);
        var lexer = new MemoryMachineAssemblyLexer(inputStream);
        var tokens = new CommonTokenStream(lexer);
        var parser = new MemoryMachineAssemblyParser(tokens);

        var tree = parser.program();
        var stringRepresentation = tree.ToStringTree(parser);

        stringRepresentation.MatchSnapshot();
    }

    [Fact(DisplayName = "Parse InsertionSort Should Create Valid AST")]
    public void ParseArrayToRegistersShouldCreateValidAST()
    {
        string input = Programs.ArrayToRegisters;

        var inputStream = new AntlrInputStream(input);
        var lexer = new MemoryMachineAssemblyLexer(inputStream);
        var tokens = new CommonTokenStream(lexer);
        var parser = new MemoryMachineAssemblyParser(tokens);

        var tree = parser.program();
        var stringRepresentation = tree.ToStringTree(parser);

        stringRepresentation.MatchSnapshot();
    }
}

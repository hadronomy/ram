//------------------------------------------------------------------------------
// <auto-generated>
//     This code was generated by a tool.
//     ANTLR Version: 4.13.1
//
//     Changes to this file may cause incorrect behavior and will be lost if
//     the code is regenerated.
// </auto-generated>
//------------------------------------------------------------------------------

// Generated from /home/hadronomy/practicas/PR2-DAA-2425/src/MemoryMachine.Core/Parser/MemoryMachineAssembly.g4 by ANTLR 4.13.1

// Unreachable code detected
#pragma warning disable 0162
// The variable '...' is assigned but its value is never used
#pragma warning disable 0219
// Missing XML comment for publicly visible type or member '...'
#pragma warning disable 1591
// Ambiguous reference in cref attribute
#pragma warning disable 419

namespace MemoryMachine.Core.Parser {
#pragma warning disable 3021

using Antlr4.Runtime.Misc;
using IErrorNode = Antlr4.Runtime.Tree.IErrorNode;
using ITerminalNode = Antlr4.Runtime.Tree.ITerminalNode;
using IToken = Antlr4.Runtime.IToken;
using ParserRuleContext = Antlr4.Runtime.ParserRuleContext;

/// <summary>
/// This class provides an empty implementation of <see cref="IMemoryMachineAssemblyListener"/>,
/// which can be extended to create a listener which only needs to handle a subset
/// of the available methods.
/// </summary>
[System.CodeDom.Compiler.GeneratedCode("ANTLR", "4.13.1")]
[System.Diagnostics.DebuggerNonUserCode]
[System.CLSCompliant(false)]
public partial class MemoryMachineAssemblyBaseListener : IMemoryMachineAssemblyListener {
	/// <summary>
	/// Enter a parse tree produced by <see cref="MemoryMachineAssemblyParser.program"/>.
	/// <para>The default implementation does nothing.</para>
	/// </summary>
	/// <param name="context">The parse tree.</param>
	public virtual void EnterProgram([NotNull] MemoryMachineAssemblyParser.ProgramContext context) { }
	/// <summary>
	/// Exit a parse tree produced by <see cref="MemoryMachineAssemblyParser.program"/>.
	/// <para>The default implementation does nothing.</para>
	/// </summary>
	/// <param name="context">The parse tree.</param>
	public virtual void ExitProgram([NotNull] MemoryMachineAssemblyParser.ProgramContext context) { }
	/// <summary>
	/// Enter a parse tree produced by <see cref="MemoryMachineAssemblyParser.line"/>.
	/// <para>The default implementation does nothing.</para>
	/// </summary>
	/// <param name="context">The parse tree.</param>
	public virtual void EnterLine([NotNull] MemoryMachineAssemblyParser.LineContext context) { }
	/// <summary>
	/// Exit a parse tree produced by <see cref="MemoryMachineAssemblyParser.line"/>.
	/// <para>The default implementation does nothing.</para>
	/// </summary>
	/// <param name="context">The parse tree.</param>
	public virtual void ExitLine([NotNull] MemoryMachineAssemblyParser.LineContext context) { }
	/// <summary>
	/// Enter a parse tree produced by <see cref="MemoryMachineAssemblyParser.label_definition"/>.
	/// <para>The default implementation does nothing.</para>
	/// </summary>
	/// <param name="context">The parse tree.</param>
	public virtual void EnterLabel_definition([NotNull] MemoryMachineAssemblyParser.Label_definitionContext context) { }
	/// <summary>
	/// Exit a parse tree produced by <see cref="MemoryMachineAssemblyParser.label_definition"/>.
	/// <para>The default implementation does nothing.</para>
	/// </summary>
	/// <param name="context">The parse tree.</param>
	public virtual void ExitLabel_definition([NotNull] MemoryMachineAssemblyParser.Label_definitionContext context) { }
	/// <summary>
	/// Enter a parse tree produced by <see cref="MemoryMachineAssemblyParser.label"/>.
	/// <para>The default implementation does nothing.</para>
	/// </summary>
	/// <param name="context">The parse tree.</param>
	public virtual void EnterLabel([NotNull] MemoryMachineAssemblyParser.LabelContext context) { }
	/// <summary>
	/// Exit a parse tree produced by <see cref="MemoryMachineAssemblyParser.label"/>.
	/// <para>The default implementation does nothing.</para>
	/// </summary>
	/// <param name="context">The parse tree.</param>
	public virtual void ExitLabel([NotNull] MemoryMachineAssemblyParser.LabelContext context) { }
	/// <summary>
	/// Enter a parse tree produced by <see cref="MemoryMachineAssemblyParser.instruction"/>.
	/// <para>The default implementation does nothing.</para>
	/// </summary>
	/// <param name="context">The parse tree.</param>
	public virtual void EnterInstruction([NotNull] MemoryMachineAssemblyParser.InstructionContext context) { }
	/// <summary>
	/// Exit a parse tree produced by <see cref="MemoryMachineAssemblyParser.instruction"/>.
	/// <para>The default implementation does nothing.</para>
	/// </summary>
	/// <param name="context">The parse tree.</param>
	public virtual void ExitInstruction([NotNull] MemoryMachineAssemblyParser.InstructionContext context) { }
	/// <summary>
	/// Enter a parse tree produced by the <c>NumberOperand</c>
	/// labeled alternative in <see cref="MemoryMachineAssemblyParser.argument"/>.
	/// <para>The default implementation does nothing.</para>
	/// </summary>
	/// <param name="context">The parse tree.</param>
	public virtual void EnterNumberOperand([NotNull] MemoryMachineAssemblyParser.NumberOperandContext context) { }
	/// <summary>
	/// Exit a parse tree produced by the <c>NumberOperand</c>
	/// labeled alternative in <see cref="MemoryMachineAssemblyParser.argument"/>.
	/// <para>The default implementation does nothing.</para>
	/// </summary>
	/// <param name="context">The parse tree.</param>
	public virtual void ExitNumberOperand([NotNull] MemoryMachineAssemblyParser.NumberOperandContext context) { }
	/// <summary>
	/// Enter a parse tree produced by the <c>IndirectOperand</c>
	/// labeled alternative in <see cref="MemoryMachineAssemblyParser.argument"/>.
	/// <para>The default implementation does nothing.</para>
	/// </summary>
	/// <param name="context">The parse tree.</param>
	public virtual void EnterIndirectOperand([NotNull] MemoryMachineAssemblyParser.IndirectOperandContext context) { }
	/// <summary>
	/// Exit a parse tree produced by the <c>IndirectOperand</c>
	/// labeled alternative in <see cref="MemoryMachineAssemblyParser.argument"/>.
	/// <para>The default implementation does nothing.</para>
	/// </summary>
	/// <param name="context">The parse tree.</param>
	public virtual void ExitIndirectOperand([NotNull] MemoryMachineAssemblyParser.IndirectOperandContext context) { }
	/// <summary>
	/// Enter a parse tree produced by the <c>ImmediateOperand</c>
	/// labeled alternative in <see cref="MemoryMachineAssemblyParser.argument"/>.
	/// <para>The default implementation does nothing.</para>
	/// </summary>
	/// <param name="context">The parse tree.</param>
	public virtual void EnterImmediateOperand([NotNull] MemoryMachineAssemblyParser.ImmediateOperandContext context) { }
	/// <summary>
	/// Exit a parse tree produced by the <c>ImmediateOperand</c>
	/// labeled alternative in <see cref="MemoryMachineAssemblyParser.argument"/>.
	/// <para>The default implementation does nothing.</para>
	/// </summary>
	/// <param name="context">The parse tree.</param>
	public virtual void ExitImmediateOperand([NotNull] MemoryMachineAssemblyParser.ImmediateOperandContext context) { }
	/// <summary>
	/// Enter a parse tree produced by the <c>LabelOperand</c>
	/// labeled alternative in <see cref="MemoryMachineAssemblyParser.argument"/>.
	/// <para>The default implementation does nothing.</para>
	/// </summary>
	/// <param name="context">The parse tree.</param>
	public virtual void EnterLabelOperand([NotNull] MemoryMachineAssemblyParser.LabelOperandContext context) { }
	/// <summary>
	/// Exit a parse tree produced by the <c>LabelOperand</c>
	/// labeled alternative in <see cref="MemoryMachineAssemblyParser.argument"/>.
	/// <para>The default implementation does nothing.</para>
	/// </summary>
	/// <param name="context">The parse tree.</param>
	public virtual void ExitLabelOperand([NotNull] MemoryMachineAssemblyParser.LabelOperandContext context) { }
	/// <summary>
	/// Enter a parse tree produced by <see cref="MemoryMachineAssemblyParser.comment"/>.
	/// <para>The default implementation does nothing.</para>
	/// </summary>
	/// <param name="context">The parse tree.</param>
	public virtual void EnterComment([NotNull] MemoryMachineAssemblyParser.CommentContext context) { }
	/// <summary>
	/// Exit a parse tree produced by <see cref="MemoryMachineAssemblyParser.comment"/>.
	/// <para>The default implementation does nothing.</para>
	/// </summary>
	/// <param name="context">The parse tree.</param>
	public virtual void ExitComment([NotNull] MemoryMachineAssemblyParser.CommentContext context) { }
	/// <summary>
	/// Enter a parse tree produced by <see cref="MemoryMachineAssemblyParser.direct"/>.
	/// <para>The default implementation does nothing.</para>
	/// </summary>
	/// <param name="context">The parse tree.</param>
	public virtual void EnterDirect([NotNull] MemoryMachineAssemblyParser.DirectContext context) { }
	/// <summary>
	/// Exit a parse tree produced by <see cref="MemoryMachineAssemblyParser.direct"/>.
	/// <para>The default implementation does nothing.</para>
	/// </summary>
	/// <param name="context">The parse tree.</param>
	public virtual void ExitDirect([NotNull] MemoryMachineAssemblyParser.DirectContext context) { }
	/// <summary>
	/// Enter a parse tree produced by <see cref="MemoryMachineAssemblyParser.indirect"/>.
	/// <para>The default implementation does nothing.</para>
	/// </summary>
	/// <param name="context">The parse tree.</param>
	public virtual void EnterIndirect([NotNull] MemoryMachineAssemblyParser.IndirectContext context) { }
	/// <summary>
	/// Exit a parse tree produced by <see cref="MemoryMachineAssemblyParser.indirect"/>.
	/// <para>The default implementation does nothing.</para>
	/// </summary>
	/// <param name="context">The parse tree.</param>
	public virtual void ExitIndirect([NotNull] MemoryMachineAssemblyParser.IndirectContext context) { }
	/// <summary>
	/// Enter a parse tree produced by <see cref="MemoryMachineAssemblyParser.immediate"/>.
	/// <para>The default implementation does nothing.</para>
	/// </summary>
	/// <param name="context">The parse tree.</param>
	public virtual void EnterImmediate([NotNull] MemoryMachineAssemblyParser.ImmediateContext context) { }
	/// <summary>
	/// Exit a parse tree produced by <see cref="MemoryMachineAssemblyParser.immediate"/>.
	/// <para>The default implementation does nothing.</para>
	/// </summary>
	/// <param name="context">The parse tree.</param>
	public virtual void ExitImmediate([NotNull] MemoryMachineAssemblyParser.ImmediateContext context) { }
	/// <summary>
	/// Enter a parse tree produced by <see cref="MemoryMachineAssemblyParser.accessor"/>.
	/// <para>The default implementation does nothing.</para>
	/// </summary>
	/// <param name="context">The parse tree.</param>
	public virtual void EnterAccessor([NotNull] MemoryMachineAssemblyParser.AccessorContext context) { }
	/// <summary>
	/// Exit a parse tree produced by <see cref="MemoryMachineAssemblyParser.accessor"/>.
	/// <para>The default implementation does nothing.</para>
	/// </summary>
	/// <param name="context">The parse tree.</param>
	public virtual void ExitAccessor([NotNull] MemoryMachineAssemblyParser.AccessorContext context) { }
	/// <summary>
	/// Enter a parse tree produced by <see cref="MemoryMachineAssemblyParser.index"/>.
	/// <para>The default implementation does nothing.</para>
	/// </summary>
	/// <param name="context">The parse tree.</param>
	public virtual void EnterIndex([NotNull] MemoryMachineAssemblyParser.IndexContext context) { }
	/// <summary>
	/// Exit a parse tree produced by <see cref="MemoryMachineAssemblyParser.index"/>.
	/// <para>The default implementation does nothing.</para>
	/// </summary>
	/// <param name="context">The parse tree.</param>
	public virtual void ExitIndex([NotNull] MemoryMachineAssemblyParser.IndexContext context) { }

	/// <inheritdoc/>
	/// <remarks>The default implementation does nothing.</remarks>
	public virtual void EnterEveryRule([NotNull] ParserRuleContext context) { }
	/// <inheritdoc/>
	/// <remarks>The default implementation does nothing.</remarks>
	public virtual void ExitEveryRule([NotNull] ParserRuleContext context) { }
	/// <inheritdoc/>
	/// <remarks>The default implementation does nothing.</remarks>
	public virtual void VisitTerminal([NotNull] ITerminalNode node) { }
	/// <inheritdoc/>
	/// <remarks>The default implementation does nothing.</remarks>
	public virtual void VisitErrorNode([NotNull] IErrorNode node) { }
}
} // namespace MemoryMachine.Core.Parser

import type { AccessorContext, CommentContext, DirectContext, ImmediateContext, ImmediateOperandContext, IndexContext, IndirectContext, IndirectOperandContext, InstructionContext, Label_definitionContext, LabelContext, LabelOperandContext, LineContext, NumberOperandContext, ProgramContext } from './MemoryMachineAssemblyParser.js';

import { AbstractParseTreeVisitor } from 'antlr4ng';

/**
 * This interface defines a complete generic visitor for a parse tree produced
 * by `MemoryMachineAssemblyParser`.
 *
 * @param <Result> The return type of the visit operation. Use `void` for
 * operations with no return type.
 */
export class MemoryMachineAssemblyVisitor<Result> extends AbstractParseTreeVisitor<Result> {
  /**
   * Visit a parse tree produced by `MemoryMachineAssemblyParser.program`.
   * @param ctx the parse tree
   * @return the visitor result
   */
  visitProgram?: (ctx: ProgramContext) => Result;
  /**
   * Visit a parse tree produced by `MemoryMachineAssemblyParser.line`.
   * @param ctx the parse tree
   * @return the visitor result
   */
  visitLine?: (ctx: LineContext) => Result;
  /**
   * Visit a parse tree produced by `MemoryMachineAssemblyParser.label_definition`.
   * @param ctx the parse tree
   * @return the visitor result
   */
  visitLabel_definition?: (ctx: Label_definitionContext) => Result;
  /**
   * Visit a parse tree produced by `MemoryMachineAssemblyParser.label`.
   * @param ctx the parse tree
   * @return the visitor result
   */
  visitLabel?: (ctx: LabelContext) => Result;
  /**
   * Visit a parse tree produced by `MemoryMachineAssemblyParser.instruction`.
   * @param ctx the parse tree
   * @return the visitor result
   */
  visitInstruction?: (ctx: InstructionContext) => Result;
  /**
   * Visit a parse tree produced by the `NumberOperand`
   * labeled alternative in `MemoryMachineAssemblyParser.argument`.
   * @param ctx the parse tree
   * @return the visitor result
   */
  visitNumberOperand?: (ctx: NumberOperandContext) => Result;
  /**
   * Visit a parse tree produced by the `IndirectOperand`
   * labeled alternative in `MemoryMachineAssemblyParser.argument`.
   * @param ctx the parse tree
   * @return the visitor result
   */
  visitIndirectOperand?: (ctx: IndirectOperandContext) => Result;
  /**
   * Visit a parse tree produced by the `ImmediateOperand`
   * labeled alternative in `MemoryMachineAssemblyParser.argument`.
   * @param ctx the parse tree
   * @return the visitor result
   */
  visitImmediateOperand?: (ctx: ImmediateOperandContext) => Result;
  /**
   * Visit a parse tree produced by the `LabelOperand`
   * labeled alternative in `MemoryMachineAssemblyParser.argument`.
   * @param ctx the parse tree
   * @return the visitor result
   */
  visitLabelOperand?: (ctx: LabelOperandContext) => Result;
  /**
   * Visit a parse tree produced by `MemoryMachineAssemblyParser.comment`.
   * @param ctx the parse tree
   * @return the visitor result
   */
  visitComment?: (ctx: CommentContext) => Result;
  /**
   * Visit a parse tree produced by `MemoryMachineAssemblyParser.direct`.
   * @param ctx the parse tree
   * @return the visitor result
   */
  visitDirect?: (ctx: DirectContext) => Result;
  /**
   * Visit a parse tree produced by `MemoryMachineAssemblyParser.indirect`.
   * @param ctx the parse tree
   * @return the visitor result
   */
  visitIndirect?: (ctx: IndirectContext) => Result;
  /**
   * Visit a parse tree produced by `MemoryMachineAssemblyParser.immediate`.
   * @param ctx the parse tree
   * @return the visitor result
   */
  visitImmediate?: (ctx: ImmediateContext) => Result;
  /**
   * Visit a parse tree produced by `MemoryMachineAssemblyParser.accessor`.
   * @param ctx the parse tree
   * @return the visitor result
   */
  visitAccessor?: (ctx: AccessorContext) => Result;
  /**
   * Visit a parse tree produced by `MemoryMachineAssemblyParser.index`.
   * @param ctx the parse tree
   * @return the visitor result
   */
  visitIndex?: (ctx: IndexContext) => Result;
}

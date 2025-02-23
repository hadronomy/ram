import type { ErrorNode, ParserRuleContext, ParseTreeListener, TerminalNode } from 'antlr4ng';

import type { AccessorContext, CommentContext, DirectContext, ImmediateContext, ImmediateOperandContext, IndexContext, IndirectContext, IndirectOperandContext, InstructionContext, Label_definitionContext, LabelContext, LabelOperandContext, LineContext, NumberOperandContext, ProgramContext } from './MemoryMachineAssemblyParser.js';

/**
 * This interface defines a complete listener for a parse tree produced by
 * `MemoryMachineAssemblyParser`.
 */
export class MemoryMachineAssemblyListener implements ParseTreeListener {
  /**
   * Enter a parse tree produced by `MemoryMachineAssemblyParser.program`.
   * @param ctx the parse tree
   */
  enterProgram?: (ctx: ProgramContext) => void;
  /**
   * Exit a parse tree produced by `MemoryMachineAssemblyParser.program`.
   * @param ctx the parse tree
   */
  exitProgram?: (ctx: ProgramContext) => void;
  /**
   * Enter a parse tree produced by `MemoryMachineAssemblyParser.line`.
   * @param ctx the parse tree
   */
  enterLine?: (ctx: LineContext) => void;
  /**
   * Exit a parse tree produced by `MemoryMachineAssemblyParser.line`.
   * @param ctx the parse tree
   */
  exitLine?: (ctx: LineContext) => void;
  /**
   * Enter a parse tree produced by `MemoryMachineAssemblyParser.label_definition`.
   * @param ctx the parse tree
   */
  enterLabel_definition?: (ctx: Label_definitionContext) => void;
  /**
   * Exit a parse tree produced by `MemoryMachineAssemblyParser.label_definition`.
   * @param ctx the parse tree
   */
  exitLabel_definition?: (ctx: Label_definitionContext) => void;
  /**
   * Enter a parse tree produced by `MemoryMachineAssemblyParser.label`.
   * @param ctx the parse tree
   */
  enterLabel?: (ctx: LabelContext) => void;
  /**
   * Exit a parse tree produced by `MemoryMachineAssemblyParser.label`.
   * @param ctx the parse tree
   */
  exitLabel?: (ctx: LabelContext) => void;
  /**
   * Enter a parse tree produced by `MemoryMachineAssemblyParser.instruction`.
   * @param ctx the parse tree
   */
  enterInstruction?: (ctx: InstructionContext) => void;
  /**
   * Exit a parse tree produced by `MemoryMachineAssemblyParser.instruction`.
   * @param ctx the parse tree
   */
  exitInstruction?: (ctx: InstructionContext) => void;
  /**
   * Enter a parse tree produced by the `NumberOperand`
   * labeled alternative in `MemoryMachineAssemblyParser.argument`.
   * @param ctx the parse tree
   */
  enterNumberOperand?: (ctx: NumberOperandContext) => void;
  /**
   * Exit a parse tree produced by the `NumberOperand`
   * labeled alternative in `MemoryMachineAssemblyParser.argument`.
   * @param ctx the parse tree
   */
  exitNumberOperand?: (ctx: NumberOperandContext) => void;
  /**
   * Enter a parse tree produced by the `IndirectOperand`
   * labeled alternative in `MemoryMachineAssemblyParser.argument`.
   * @param ctx the parse tree
   */
  enterIndirectOperand?: (ctx: IndirectOperandContext) => void;
  /**
   * Exit a parse tree produced by the `IndirectOperand`
   * labeled alternative in `MemoryMachineAssemblyParser.argument`.
   * @param ctx the parse tree
   */
  exitIndirectOperand?: (ctx: IndirectOperandContext) => void;
  /**
   * Enter a parse tree produced by the `ImmediateOperand`
   * labeled alternative in `MemoryMachineAssemblyParser.argument`.
   * @param ctx the parse tree
   */
  enterImmediateOperand?: (ctx: ImmediateOperandContext) => void;
  /**
   * Exit a parse tree produced by the `ImmediateOperand`
   * labeled alternative in `MemoryMachineAssemblyParser.argument`.
   * @param ctx the parse tree
   */
  exitImmediateOperand?: (ctx: ImmediateOperandContext) => void;
  /**
   * Enter a parse tree produced by the `LabelOperand`
   * labeled alternative in `MemoryMachineAssemblyParser.argument`.
   * @param ctx the parse tree
   */
  enterLabelOperand?: (ctx: LabelOperandContext) => void;
  /**
   * Exit a parse tree produced by the `LabelOperand`
   * labeled alternative in `MemoryMachineAssemblyParser.argument`.
   * @param ctx the parse tree
   */
  exitLabelOperand?: (ctx: LabelOperandContext) => void;
  /**
   * Enter a parse tree produced by `MemoryMachineAssemblyParser.comment`.
   * @param ctx the parse tree
   */
  enterComment?: (ctx: CommentContext) => void;
  /**
   * Exit a parse tree produced by `MemoryMachineAssemblyParser.comment`.
   * @param ctx the parse tree
   */
  exitComment?: (ctx: CommentContext) => void;
  /**
   * Enter a parse tree produced by `MemoryMachineAssemblyParser.direct`.
   * @param ctx the parse tree
   */
  enterDirect?: (ctx: DirectContext) => void;
  /**
   * Exit a parse tree produced by `MemoryMachineAssemblyParser.direct`.
   * @param ctx the parse tree
   */
  exitDirect?: (ctx: DirectContext) => void;
  /**
   * Enter a parse tree produced by `MemoryMachineAssemblyParser.indirect`.
   * @param ctx the parse tree
   */
  enterIndirect?: (ctx: IndirectContext) => void;
  /**
   * Exit a parse tree produced by `MemoryMachineAssemblyParser.indirect`.
   * @param ctx the parse tree
   */
  exitIndirect?: (ctx: IndirectContext) => void;
  /**
   * Enter a parse tree produced by `MemoryMachineAssemblyParser.immediate`.
   * @param ctx the parse tree
   */
  enterImmediate?: (ctx: ImmediateContext) => void;
  /**
   * Exit a parse tree produced by `MemoryMachineAssemblyParser.immediate`.
   * @param ctx the parse tree
   */
  exitImmediate?: (ctx: ImmediateContext) => void;
  /**
   * Enter a parse tree produced by `MemoryMachineAssemblyParser.accessor`.
   * @param ctx the parse tree
   */
  enterAccessor?: (ctx: AccessorContext) => void;
  /**
   * Exit a parse tree produced by `MemoryMachineAssemblyParser.accessor`.
   * @param ctx the parse tree
   */
  exitAccessor?: (ctx: AccessorContext) => void;
  /**
   * Enter a parse tree produced by `MemoryMachineAssemblyParser.index`.
   * @param ctx the parse tree
   */
  enterIndex?: (ctx: IndexContext) => void;
  /**
   * Exit a parse tree produced by `MemoryMachineAssemblyParser.index`.
   * @param ctx the parse tree
   */
  exitIndex?: (ctx: IndexContext) => void;

  visitTerminal(node: TerminalNode): void {}
  visitErrorNode(node: ErrorNode): void {}
  enterEveryRule(node: ParserRuleContext): void {}
  exitEveryRule(node: ParserRuleContext): void {}
}

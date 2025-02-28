
import * as antlr from "antlr4ng";
import { Token } from "antlr4ng";

import { MemoryMachineAssemblyListener } from "./MemoryMachineAssemblyListener.js";
import { MemoryMachineAssemblyVisitor } from "./MemoryMachineAssemblyVisitor.js";

// for running tests with parameters, TODO: discuss strategy for typed parameters in CI
// eslint-disable-next-line no-unused-vars
type int = number;


export class MemoryMachineAssemblyParser extends antlr.Parser {
    public static readonly IDENTIFIER = 1;
    public static readonly NUMBER = 2;
    public static readonly COLON = 3;
    public static readonly EQUALS = 4;
    public static readonly ASTERISK = 5;
    public static readonly LBRACKET = 6;
    public static readonly RBRACKET = 7;
    public static readonly WS = 8;
    public static readonly COMMENT = 9;
    public static readonly NEWLINE = 10;
    public static readonly RULE_program = 0;
    public static readonly RULE_line = 1;
    public static readonly RULE_label_definition = 2;
    public static readonly RULE_label = 3;
    public static readonly RULE_instruction = 4;
    public static readonly RULE_argument = 5;
    public static readonly RULE_comment = 6;
    public static readonly RULE_direct = 7;
    public static readonly RULE_indirect = 8;
    public static readonly RULE_immediate = 9;
    public static readonly RULE_accessor = 10;
    public static readonly RULE_index = 11;

    public static readonly literalNames = [
        null, null, null, "':'", "'='", "'*'", "'['", "']'"
    ];

    public static readonly symbolicNames = [
        null, "IDENTIFIER", "NUMBER", "COLON", "EQUALS", "ASTERISK", "LBRACKET", 
        "RBRACKET", "WS", "COMMENT", "NEWLINE"
    ];
    public static readonly ruleNames = [
        "program", "line", "label_definition", "label", "instruction", "argument", 
        "comment", "direct", "indirect", "immediate", "accessor", "index",
    ];

    public get grammarFileName(): string { return "MemoryMachineAssembly.g4"; }
    public get literalNames(): (string | null)[] { return MemoryMachineAssemblyParser.literalNames; }
    public get symbolicNames(): (string | null)[] { return MemoryMachineAssemblyParser.symbolicNames; }
    public get ruleNames(): string[] { return MemoryMachineAssemblyParser.ruleNames; }
    public get serializedATN(): number[] { return MemoryMachineAssemblyParser._serializedATN; }

    protected createFailedPredicateException(predicate?: string, message?: string): antlr.FailedPredicateException {
        return new antlr.FailedPredicateException(this, predicate, message);
    }

    public constructor(input: antlr.TokenStream) {
        super(input);
        this.interpreter = new antlr.ParserATNSimulator(this, MemoryMachineAssemblyParser._ATN, MemoryMachineAssemblyParser.decisionsToDFA, new antlr.PredictionContextCache());
    }
    public program(): ProgramContext {
        let localContext = new ProgramContext(this.context, this.state);
        this.enterRule(localContext, 0, MemoryMachineAssemblyParser.RULE_program);
        let _la: number;
        try {
            let alternative: number;
            this.enterOuterAlt(localContext, 1);
            {
            this.state = 24;
            this.line();
            this.state = 33;
            this.errorHandler.sync(this);
            alternative = this.interpreter.adaptivePredict(this.tokenStream, 1, this.context);
            while (alternative !== 2 && alternative !== antlr.ATN.INVALID_ALT_NUMBER) {
                if (alternative === 1) {
                    {
                    {
                    this.state = 26;
                    this.errorHandler.sync(this);
                    _la = this.tokenStream.LA(1);
                    do {
                        {
                        {
                        this.state = 25;
                        this.match(MemoryMachineAssemblyParser.NEWLINE);
                        }
                        }
                        this.state = 28;
                        this.errorHandler.sync(this);
                        _la = this.tokenStream.LA(1);
                    } while (_la === 10);
                    this.state = 30;
                    this.line();
                    }
                    }
                }
                this.state = 35;
                this.errorHandler.sync(this);
                alternative = this.interpreter.adaptivePredict(this.tokenStream, 1, this.context);
            }
            this.state = 39;
            this.errorHandler.sync(this);
            _la = this.tokenStream.LA(1);
            while (_la === 10) {
                {
                {
                this.state = 36;
                this.match(MemoryMachineAssemblyParser.NEWLINE);
                }
                }
                this.state = 41;
                this.errorHandler.sync(this);
                _la = this.tokenStream.LA(1);
            }
            this.state = 42;
            this.match(MemoryMachineAssemblyParser.EOF);
            }
        }
        catch (re) {
            if (re instanceof antlr.RecognitionException) {
                this.errorHandler.reportError(this, re);
                this.errorHandler.recover(this, re);
            } else {
                throw re;
            }
        }
        finally {
            this.exitRule();
        }
        return localContext;
    }
    public line(): LineContext {
        let localContext = new LineContext(this.context, this.state);
        this.enterRule(localContext, 2, MemoryMachineAssemblyParser.RULE_line);
        let _la: number;
        try {
            this.state = 50;
            this.errorHandler.sync(this);
            switch (this.interpreter.adaptivePredict(this.tokenStream, 4, this.context) ) {
            case 1:
                this.enterOuterAlt(localContext, 1);
                {
                this.state = 44;
                this.instruction();
                this.state = 46;
                this.errorHandler.sync(this);
                _la = this.tokenStream.LA(1);
                if (_la === 9) {
                    {
                    this.state = 45;
                    this.comment();
                    }
                }

                }
                break;
            case 2:
                this.enterOuterAlt(localContext, 2);
                {
                this.state = 48;
                this.label_definition();
                }
                break;
            case 3:
                this.enterOuterAlt(localContext, 3);
                {
                this.state = 49;
                this.comment();
                }
                break;
            }
        }
        catch (re) {
            if (re instanceof antlr.RecognitionException) {
                this.errorHandler.reportError(this, re);
                this.errorHandler.recover(this, re);
            } else {
                throw re;
            }
        }
        finally {
            this.exitRule();
        }
        return localContext;
    }
    public label_definition(): Label_definitionContext {
        let localContext = new Label_definitionContext(this.context, this.state);
        this.enterRule(localContext, 4, MemoryMachineAssemblyParser.RULE_label_definition);
        let _la: number;
        try {
            this.enterOuterAlt(localContext, 1);
            {
            this.state = 52;
            this.match(MemoryMachineAssemblyParser.IDENTIFIER);
            this.state = 53;
            this.match(MemoryMachineAssemblyParser.COLON);
            this.state = 55;
            this.errorHandler.sync(this);
            _la = this.tokenStream.LA(1);
            if (_la === 10) {
                {
                this.state = 54;
                this.match(MemoryMachineAssemblyParser.NEWLINE);
                }
            }

            this.state = 57;
            this.instruction();
            this.state = 59;
            this.errorHandler.sync(this);
            _la = this.tokenStream.LA(1);
            if (_la === 9) {
                {
                this.state = 58;
                this.comment();
                }
            }

            }
        }
        catch (re) {
            if (re instanceof antlr.RecognitionException) {
                this.errorHandler.reportError(this, re);
                this.errorHandler.recover(this, re);
            } else {
                throw re;
            }
        }
        finally {
            this.exitRule();
        }
        return localContext;
    }
    public label(): LabelContext {
        let localContext = new LabelContext(this.context, this.state);
        this.enterRule(localContext, 6, MemoryMachineAssemblyParser.RULE_label);
        try {
            this.enterOuterAlt(localContext, 1);
            {
            this.state = 61;
            this.match(MemoryMachineAssemblyParser.IDENTIFIER);
            }
        }
        catch (re) {
            if (re instanceof antlr.RecognitionException) {
                this.errorHandler.reportError(this, re);
                this.errorHandler.recover(this, re);
            } else {
                throw re;
            }
        }
        finally {
            this.exitRule();
        }
        return localContext;
    }
    public instruction(): InstructionContext {
        let localContext = new InstructionContext(this.context, this.state);
        this.enterRule(localContext, 8, MemoryMachineAssemblyParser.RULE_instruction);
        let _la: number;
        try {
            this.enterOuterAlt(localContext, 1);
            {
            this.state = 63;
            this.match(MemoryMachineAssemblyParser.IDENTIFIER);
            this.state = 65;
            this.errorHandler.sync(this);
            _la = this.tokenStream.LA(1);
            if ((((_la) & ~0x1F) === 0 && ((1 << _la) & 54) !== 0)) {
                {
                this.state = 64;
                this.argument();
                }
            }

            }
        }
        catch (re) {
            if (re instanceof antlr.RecognitionException) {
                this.errorHandler.reportError(this, re);
                this.errorHandler.recover(this, re);
            } else {
                throw re;
            }
        }
        finally {
            this.exitRule();
        }
        return localContext;
    }
    public argument(): ArgumentContext {
        let localContext = new ArgumentContext(this.context, this.state);
        this.enterRule(localContext, 10, MemoryMachineAssemblyParser.RULE_argument);
        try {
            this.state = 71;
            this.errorHandler.sync(this);
            switch (this.tokenStream.LA(1)) {
            case MemoryMachineAssemblyParser.NUMBER:
                localContext = new NumberOperandContext(localContext);
                this.enterOuterAlt(localContext, 1);
                {
                this.state = 67;
                this.direct();
                }
                break;
            case MemoryMachineAssemblyParser.ASTERISK:
                localContext = new IndirectOperandContext(localContext);
                this.enterOuterAlt(localContext, 2);
                {
                this.state = 68;
                this.indirect();
                }
                break;
            case MemoryMachineAssemblyParser.EQUALS:
                localContext = new ImmediateOperandContext(localContext);
                this.enterOuterAlt(localContext, 3);
                {
                this.state = 69;
                this.immediate();
                }
                break;
            case MemoryMachineAssemblyParser.IDENTIFIER:
                localContext = new LabelOperandContext(localContext);
                this.enterOuterAlt(localContext, 4);
                {
                this.state = 70;
                this.label();
                }
                break;
            default:
                throw new antlr.NoViableAltException(this);
            }
        }
        catch (re) {
            if (re instanceof antlr.RecognitionException) {
                this.errorHandler.reportError(this, re);
                this.errorHandler.recover(this, re);
            } else {
                throw re;
            }
        }
        finally {
            this.exitRule();
        }
        return localContext;
    }
    public comment(): CommentContext {
        let localContext = new CommentContext(this.context, this.state);
        this.enterRule(localContext, 12, MemoryMachineAssemblyParser.RULE_comment);
        try {
            this.enterOuterAlt(localContext, 1);
            {
            this.state = 73;
            this.match(MemoryMachineAssemblyParser.COMMENT);
            }
        }
        catch (re) {
            if (re instanceof antlr.RecognitionException) {
                this.errorHandler.reportError(this, re);
                this.errorHandler.recover(this, re);
            } else {
                throw re;
            }
        }
        finally {
            this.exitRule();
        }
        return localContext;
    }
    public direct(): DirectContext {
        let localContext = new DirectContext(this.context, this.state);
        this.enterRule(localContext, 14, MemoryMachineAssemblyParser.RULE_direct);
        let _la: number;
        try {
            this.enterOuterAlt(localContext, 1);
            {
            this.state = 75;
            this.match(MemoryMachineAssemblyParser.NUMBER);
            this.state = 77;
            this.errorHandler.sync(this);
            _la = this.tokenStream.LA(1);
            if (_la === 6) {
                {
                this.state = 76;
                this.accessor();
                }
            }

            }
        }
        catch (re) {
            if (re instanceof antlr.RecognitionException) {
                this.errorHandler.reportError(this, re);
                this.errorHandler.recover(this, re);
            } else {
                throw re;
            }
        }
        finally {
            this.exitRule();
        }
        return localContext;
    }
    public indirect(): IndirectContext {
        let localContext = new IndirectContext(this.context, this.state);
        this.enterRule(localContext, 16, MemoryMachineAssemblyParser.RULE_indirect);
        try {
            this.enterOuterAlt(localContext, 1);
            {
            this.state = 79;
            this.match(MemoryMachineAssemblyParser.ASTERISK);
            this.state = 80;
            this.match(MemoryMachineAssemblyParser.NUMBER);
            }
        }
        catch (re) {
            if (re instanceof antlr.RecognitionException) {
                this.errorHandler.reportError(this, re);
                this.errorHandler.recover(this, re);
            } else {
                throw re;
            }
        }
        finally {
            this.exitRule();
        }
        return localContext;
    }
    public immediate(): ImmediateContext {
        let localContext = new ImmediateContext(this.context, this.state);
        this.enterRule(localContext, 18, MemoryMachineAssemblyParser.RULE_immediate);
        try {
            this.enterOuterAlt(localContext, 1);
            {
            this.state = 82;
            this.match(MemoryMachineAssemblyParser.EQUALS);
            this.state = 83;
            this.match(MemoryMachineAssemblyParser.NUMBER);
            }
        }
        catch (re) {
            if (re instanceof antlr.RecognitionException) {
                this.errorHandler.reportError(this, re);
                this.errorHandler.recover(this, re);
            } else {
                throw re;
            }
        }
        finally {
            this.exitRule();
        }
        return localContext;
    }
    public accessor(): AccessorContext {
        let localContext = new AccessorContext(this.context, this.state);
        this.enterRule(localContext, 20, MemoryMachineAssemblyParser.RULE_accessor);
        try {
            this.enterOuterAlt(localContext, 1);
            {
            this.state = 85;
            this.match(MemoryMachineAssemblyParser.LBRACKET);
            this.state = 86;
            this.index();
            this.state = 87;
            this.match(MemoryMachineAssemblyParser.RBRACKET);
            }
        }
        catch (re) {
            if (re instanceof antlr.RecognitionException) {
                this.errorHandler.reportError(this, re);
                this.errorHandler.recover(this, re);
            } else {
                throw re;
            }
        }
        finally {
            this.exitRule();
        }
        return localContext;
    }
    public index(): IndexContext {
        let localContext = new IndexContext(this.context, this.state);
        this.enterRule(localContext, 22, MemoryMachineAssemblyParser.RULE_index);
        try {
            this.state = 92;
            this.errorHandler.sync(this);
            switch (this.tokenStream.LA(1)) {
            case MemoryMachineAssemblyParser.NUMBER:
                this.enterOuterAlt(localContext, 1);
                {
                this.state = 89;
                this.direct();
                }
                break;
            case MemoryMachineAssemblyParser.ASTERISK:
                this.enterOuterAlt(localContext, 2);
                {
                this.state = 90;
                this.indirect();
                }
                break;
            case MemoryMachineAssemblyParser.EQUALS:
                this.enterOuterAlt(localContext, 3);
                {
                this.state = 91;
                this.immediate();
                }
                break;
            default:
                throw new antlr.NoViableAltException(this);
            }
        }
        catch (re) {
            if (re instanceof antlr.RecognitionException) {
                this.errorHandler.reportError(this, re);
                this.errorHandler.recover(this, re);
            } else {
                throw re;
            }
        }
        finally {
            this.exitRule();
        }
        return localContext;
    }

    public static readonly _serializedATN: number[] = [
        4,1,10,95,2,0,7,0,2,1,7,1,2,2,7,2,2,3,7,3,2,4,7,4,2,5,7,5,2,6,7,
        6,2,7,7,7,2,8,7,8,2,9,7,9,2,10,7,10,2,11,7,11,1,0,1,0,4,0,27,8,0,
        11,0,12,0,28,1,0,5,0,32,8,0,10,0,12,0,35,9,0,1,0,5,0,38,8,0,10,0,
        12,0,41,9,0,1,0,1,0,1,1,1,1,3,1,47,8,1,1,1,1,1,3,1,51,8,1,1,2,1,
        2,1,2,3,2,56,8,2,1,2,1,2,3,2,60,8,2,1,3,1,3,1,4,1,4,3,4,66,8,4,1,
        5,1,5,1,5,1,5,3,5,72,8,5,1,6,1,6,1,7,1,7,3,7,78,8,7,1,8,1,8,1,8,
        1,9,1,9,1,9,1,10,1,10,1,10,1,10,1,11,1,11,1,11,3,11,93,8,11,1,11,
        0,0,12,0,2,4,6,8,10,12,14,16,18,20,22,0,0,97,0,24,1,0,0,0,2,50,1,
        0,0,0,4,52,1,0,0,0,6,61,1,0,0,0,8,63,1,0,0,0,10,71,1,0,0,0,12,73,
        1,0,0,0,14,75,1,0,0,0,16,79,1,0,0,0,18,82,1,0,0,0,20,85,1,0,0,0,
        22,92,1,0,0,0,24,33,3,2,1,0,25,27,5,10,0,0,26,25,1,0,0,0,27,28,1,
        0,0,0,28,26,1,0,0,0,28,29,1,0,0,0,29,30,1,0,0,0,30,32,3,2,1,0,31,
        26,1,0,0,0,32,35,1,0,0,0,33,31,1,0,0,0,33,34,1,0,0,0,34,39,1,0,0,
        0,35,33,1,0,0,0,36,38,5,10,0,0,37,36,1,0,0,0,38,41,1,0,0,0,39,37,
        1,0,0,0,39,40,1,0,0,0,40,42,1,0,0,0,41,39,1,0,0,0,42,43,5,0,0,1,
        43,1,1,0,0,0,44,46,3,8,4,0,45,47,3,12,6,0,46,45,1,0,0,0,46,47,1,
        0,0,0,47,51,1,0,0,0,48,51,3,4,2,0,49,51,3,12,6,0,50,44,1,0,0,0,50,
        48,1,0,0,0,50,49,1,0,0,0,51,3,1,0,0,0,52,53,5,1,0,0,53,55,5,3,0,
        0,54,56,5,10,0,0,55,54,1,0,0,0,55,56,1,0,0,0,56,57,1,0,0,0,57,59,
        3,8,4,0,58,60,3,12,6,0,59,58,1,0,0,0,59,60,1,0,0,0,60,5,1,0,0,0,
        61,62,5,1,0,0,62,7,1,0,0,0,63,65,5,1,0,0,64,66,3,10,5,0,65,64,1,
        0,0,0,65,66,1,0,0,0,66,9,1,0,0,0,67,72,3,14,7,0,68,72,3,16,8,0,69,
        72,3,18,9,0,70,72,3,6,3,0,71,67,1,0,0,0,71,68,1,0,0,0,71,69,1,0,
        0,0,71,70,1,0,0,0,72,11,1,0,0,0,73,74,5,9,0,0,74,13,1,0,0,0,75,77,
        5,2,0,0,76,78,3,20,10,0,77,76,1,0,0,0,77,78,1,0,0,0,78,15,1,0,0,
        0,79,80,5,5,0,0,80,81,5,2,0,0,81,17,1,0,0,0,82,83,5,4,0,0,83,84,
        5,2,0,0,84,19,1,0,0,0,85,86,5,6,0,0,86,87,3,22,11,0,87,88,5,7,0,
        0,88,21,1,0,0,0,89,93,3,14,7,0,90,93,3,16,8,0,91,93,3,18,9,0,92,
        89,1,0,0,0,92,90,1,0,0,0,92,91,1,0,0,0,93,23,1,0,0,0,11,28,33,39,
        46,50,55,59,65,71,77,92
    ];

    private static __ATN: antlr.ATN;
    public static get _ATN(): antlr.ATN {
        if (!MemoryMachineAssemblyParser.__ATN) {
            MemoryMachineAssemblyParser.__ATN = new antlr.ATNDeserializer().deserialize(MemoryMachineAssemblyParser._serializedATN);
        }

        return MemoryMachineAssemblyParser.__ATN;
    }


    private static readonly vocabulary = new antlr.Vocabulary(MemoryMachineAssemblyParser.literalNames, MemoryMachineAssemblyParser.symbolicNames, []);

    public override get vocabulary(): antlr.Vocabulary {
        return MemoryMachineAssemblyParser.vocabulary;
    }

    private static readonly decisionsToDFA = MemoryMachineAssemblyParser._ATN.decisionToState.map( (ds: antlr.DecisionState, index: number) => new antlr.DFA(ds, index) );
}

export class ProgramContext extends antlr.ParserRuleContext {
    public constructor(parent: antlr.ParserRuleContext | null, invokingState: number) {
        super(parent, invokingState);
    }
    public line(): LineContext[];
    public line(i: number): LineContext | null;
    public line(i?: number): LineContext[] | LineContext | null {
        if (i === undefined) {
            return this.getRuleContexts(LineContext);
        }

        return this.getRuleContext(i, LineContext);
    }
    public EOF(): antlr.TerminalNode {
        return this.getToken(MemoryMachineAssemblyParser.EOF, 0)!;
    }
    public NEWLINE(): antlr.TerminalNode[];
    public NEWLINE(i: number): antlr.TerminalNode | null;
    public NEWLINE(i?: number): antlr.TerminalNode | null | antlr.TerminalNode[] {
    	if (i === undefined) {
    		return this.getTokens(MemoryMachineAssemblyParser.NEWLINE);
    	} else {
    		return this.getToken(MemoryMachineAssemblyParser.NEWLINE, i);
    	}
    }
    public override get ruleIndex(): number {
        return MemoryMachineAssemblyParser.RULE_program;
    }
    public override enterRule(listener: MemoryMachineAssemblyListener): void {
        if(listener.enterProgram) {
             listener.enterProgram(this);
        }
    }
    public override exitRule(listener: MemoryMachineAssemblyListener): void {
        if(listener.exitProgram) {
             listener.exitProgram(this);
        }
    }
    public override accept<Result>(visitor: MemoryMachineAssemblyVisitor<Result>): Result | null {
        if (visitor.visitProgram) {
            return visitor.visitProgram(this);
        } else {
            return visitor.visitChildren(this);
        }
    }
}


export class LineContext extends antlr.ParserRuleContext {
    public constructor(parent: antlr.ParserRuleContext | null, invokingState: number) {
        super(parent, invokingState);
    }
    public instruction(): InstructionContext | null {
        return this.getRuleContext(0, InstructionContext);
    }
    public comment(): CommentContext | null {
        return this.getRuleContext(0, CommentContext);
    }
    public label_definition(): Label_definitionContext | null {
        return this.getRuleContext(0, Label_definitionContext);
    }
    public override get ruleIndex(): number {
        return MemoryMachineAssemblyParser.RULE_line;
    }
    public override enterRule(listener: MemoryMachineAssemblyListener): void {
        if(listener.enterLine) {
             listener.enterLine(this);
        }
    }
    public override exitRule(listener: MemoryMachineAssemblyListener): void {
        if(listener.exitLine) {
             listener.exitLine(this);
        }
    }
    public override accept<Result>(visitor: MemoryMachineAssemblyVisitor<Result>): Result | null {
        if (visitor.visitLine) {
            return visitor.visitLine(this);
        } else {
            return visitor.visitChildren(this);
        }
    }
}


export class Label_definitionContext extends antlr.ParserRuleContext {
    public constructor(parent: antlr.ParserRuleContext | null, invokingState: number) {
        super(parent, invokingState);
    }
    public IDENTIFIER(): antlr.TerminalNode {
        return this.getToken(MemoryMachineAssemblyParser.IDENTIFIER, 0)!;
    }
    public COLON(): antlr.TerminalNode {
        return this.getToken(MemoryMachineAssemblyParser.COLON, 0)!;
    }
    public instruction(): InstructionContext {
        return this.getRuleContext(0, InstructionContext)!;
    }
    public NEWLINE(): antlr.TerminalNode | null {
        return this.getToken(MemoryMachineAssemblyParser.NEWLINE, 0);
    }
    public comment(): CommentContext | null {
        return this.getRuleContext(0, CommentContext);
    }
    public override get ruleIndex(): number {
        return MemoryMachineAssemblyParser.RULE_label_definition;
    }
    public override enterRule(listener: MemoryMachineAssemblyListener): void {
        if(listener.enterLabel_definition) {
             listener.enterLabel_definition(this);
        }
    }
    public override exitRule(listener: MemoryMachineAssemblyListener): void {
        if(listener.exitLabel_definition) {
             listener.exitLabel_definition(this);
        }
    }
    public override accept<Result>(visitor: MemoryMachineAssemblyVisitor<Result>): Result | null {
        if (visitor.visitLabel_definition) {
            return visitor.visitLabel_definition(this);
        } else {
            return visitor.visitChildren(this);
        }
    }
}


export class LabelContext extends antlr.ParserRuleContext {
    public constructor(parent: antlr.ParserRuleContext | null, invokingState: number) {
        super(parent, invokingState);
    }
    public IDENTIFIER(): antlr.TerminalNode {
        return this.getToken(MemoryMachineAssemblyParser.IDENTIFIER, 0)!;
    }
    public override get ruleIndex(): number {
        return MemoryMachineAssemblyParser.RULE_label;
    }
    public override enterRule(listener: MemoryMachineAssemblyListener): void {
        if(listener.enterLabel) {
             listener.enterLabel(this);
        }
    }
    public override exitRule(listener: MemoryMachineAssemblyListener): void {
        if(listener.exitLabel) {
             listener.exitLabel(this);
        }
    }
    public override accept<Result>(visitor: MemoryMachineAssemblyVisitor<Result>): Result | null {
        if (visitor.visitLabel) {
            return visitor.visitLabel(this);
        } else {
            return visitor.visitChildren(this);
        }
    }
}


export class InstructionContext extends antlr.ParserRuleContext {
    public constructor(parent: antlr.ParserRuleContext | null, invokingState: number) {
        super(parent, invokingState);
    }
    public IDENTIFIER(): antlr.TerminalNode {
        return this.getToken(MemoryMachineAssemblyParser.IDENTIFIER, 0)!;
    }
    public argument(): ArgumentContext | null {
        return this.getRuleContext(0, ArgumentContext);
    }
    public override get ruleIndex(): number {
        return MemoryMachineAssemblyParser.RULE_instruction;
    }
    public override enterRule(listener: MemoryMachineAssemblyListener): void {
        if(listener.enterInstruction) {
             listener.enterInstruction(this);
        }
    }
    public override exitRule(listener: MemoryMachineAssemblyListener): void {
        if(listener.exitInstruction) {
             listener.exitInstruction(this);
        }
    }
    public override accept<Result>(visitor: MemoryMachineAssemblyVisitor<Result>): Result | null {
        if (visitor.visitInstruction) {
            return visitor.visitInstruction(this);
        } else {
            return visitor.visitChildren(this);
        }
    }
}


export class ArgumentContext extends antlr.ParserRuleContext {
    public constructor(parent: antlr.ParserRuleContext | null, invokingState: number) {
        super(parent, invokingState);
    }
    public override get ruleIndex(): number {
        return MemoryMachineAssemblyParser.RULE_argument;
    }
    public override copyFrom(ctx: ArgumentContext): void {
        super.copyFrom(ctx);
    }
}
export class NumberOperandContext extends ArgumentContext {
    public constructor(ctx: ArgumentContext) {
        super(ctx.parent, ctx.invokingState);
        super.copyFrom(ctx);
    }
    public direct(): DirectContext {
        return this.getRuleContext(0, DirectContext)!;
    }
    public override enterRule(listener: MemoryMachineAssemblyListener): void {
        if(listener.enterNumberOperand) {
             listener.enterNumberOperand(this);
        }
    }
    public override exitRule(listener: MemoryMachineAssemblyListener): void {
        if(listener.exitNumberOperand) {
             listener.exitNumberOperand(this);
        }
    }
    public override accept<Result>(visitor: MemoryMachineAssemblyVisitor<Result>): Result | null {
        if (visitor.visitNumberOperand) {
            return visitor.visitNumberOperand(this);
        } else {
            return visitor.visitChildren(this);
        }
    }
}
export class IndirectOperandContext extends ArgumentContext {
    public constructor(ctx: ArgumentContext) {
        super(ctx.parent, ctx.invokingState);
        super.copyFrom(ctx);
    }
    public indirect(): IndirectContext {
        return this.getRuleContext(0, IndirectContext)!;
    }
    public override enterRule(listener: MemoryMachineAssemblyListener): void {
        if(listener.enterIndirectOperand) {
             listener.enterIndirectOperand(this);
        }
    }
    public override exitRule(listener: MemoryMachineAssemblyListener): void {
        if(listener.exitIndirectOperand) {
             listener.exitIndirectOperand(this);
        }
    }
    public override accept<Result>(visitor: MemoryMachineAssemblyVisitor<Result>): Result | null {
        if (visitor.visitIndirectOperand) {
            return visitor.visitIndirectOperand(this);
        } else {
            return visitor.visitChildren(this);
        }
    }
}
export class ImmediateOperandContext extends ArgumentContext {
    public constructor(ctx: ArgumentContext) {
        super(ctx.parent, ctx.invokingState);
        super.copyFrom(ctx);
    }
    public immediate(): ImmediateContext {
        return this.getRuleContext(0, ImmediateContext)!;
    }
    public override enterRule(listener: MemoryMachineAssemblyListener): void {
        if(listener.enterImmediateOperand) {
             listener.enterImmediateOperand(this);
        }
    }
    public override exitRule(listener: MemoryMachineAssemblyListener): void {
        if(listener.exitImmediateOperand) {
             listener.exitImmediateOperand(this);
        }
    }
    public override accept<Result>(visitor: MemoryMachineAssemblyVisitor<Result>): Result | null {
        if (visitor.visitImmediateOperand) {
            return visitor.visitImmediateOperand(this);
        } else {
            return visitor.visitChildren(this);
        }
    }
}
export class LabelOperandContext extends ArgumentContext {
    public constructor(ctx: ArgumentContext) {
        super(ctx.parent, ctx.invokingState);
        super.copyFrom(ctx);
    }
    public label(): LabelContext {
        return this.getRuleContext(0, LabelContext)!;
    }
    public override enterRule(listener: MemoryMachineAssemblyListener): void {
        if(listener.enterLabelOperand) {
             listener.enterLabelOperand(this);
        }
    }
    public override exitRule(listener: MemoryMachineAssemblyListener): void {
        if(listener.exitLabelOperand) {
             listener.exitLabelOperand(this);
        }
    }
    public override accept<Result>(visitor: MemoryMachineAssemblyVisitor<Result>): Result | null {
        if (visitor.visitLabelOperand) {
            return visitor.visitLabelOperand(this);
        } else {
            return visitor.visitChildren(this);
        }
    }
}


export class CommentContext extends antlr.ParserRuleContext {
    public constructor(parent: antlr.ParserRuleContext | null, invokingState: number) {
        super(parent, invokingState);
    }
    public COMMENT(): antlr.TerminalNode {
        return this.getToken(MemoryMachineAssemblyParser.COMMENT, 0)!;
    }
    public override get ruleIndex(): number {
        return MemoryMachineAssemblyParser.RULE_comment;
    }
    public override enterRule(listener: MemoryMachineAssemblyListener): void {
        if(listener.enterComment) {
             listener.enterComment(this);
        }
    }
    public override exitRule(listener: MemoryMachineAssemblyListener): void {
        if(listener.exitComment) {
             listener.exitComment(this);
        }
    }
    public override accept<Result>(visitor: MemoryMachineAssemblyVisitor<Result>): Result | null {
        if (visitor.visitComment) {
            return visitor.visitComment(this);
        } else {
            return visitor.visitChildren(this);
        }
    }
}


export class DirectContext extends antlr.ParserRuleContext {
    public constructor(parent: antlr.ParserRuleContext | null, invokingState: number) {
        super(parent, invokingState);
    }
    public NUMBER(): antlr.TerminalNode {
        return this.getToken(MemoryMachineAssemblyParser.NUMBER, 0)!;
    }
    public accessor(): AccessorContext | null {
        return this.getRuleContext(0, AccessorContext);
    }
    public override get ruleIndex(): number {
        return MemoryMachineAssemblyParser.RULE_direct;
    }
    public override enterRule(listener: MemoryMachineAssemblyListener): void {
        if(listener.enterDirect) {
             listener.enterDirect(this);
        }
    }
    public override exitRule(listener: MemoryMachineAssemblyListener): void {
        if(listener.exitDirect) {
             listener.exitDirect(this);
        }
    }
    public override accept<Result>(visitor: MemoryMachineAssemblyVisitor<Result>): Result | null {
        if (visitor.visitDirect) {
            return visitor.visitDirect(this);
        } else {
            return visitor.visitChildren(this);
        }
    }
}


export class IndirectContext extends antlr.ParserRuleContext {
    public constructor(parent: antlr.ParserRuleContext | null, invokingState: number) {
        super(parent, invokingState);
    }
    public ASTERISK(): antlr.TerminalNode {
        return this.getToken(MemoryMachineAssemblyParser.ASTERISK, 0)!;
    }
    public NUMBER(): antlr.TerminalNode {
        return this.getToken(MemoryMachineAssemblyParser.NUMBER, 0)!;
    }
    public override get ruleIndex(): number {
        return MemoryMachineAssemblyParser.RULE_indirect;
    }
    public override enterRule(listener: MemoryMachineAssemblyListener): void {
        if(listener.enterIndirect) {
             listener.enterIndirect(this);
        }
    }
    public override exitRule(listener: MemoryMachineAssemblyListener): void {
        if(listener.exitIndirect) {
             listener.exitIndirect(this);
        }
    }
    public override accept<Result>(visitor: MemoryMachineAssemblyVisitor<Result>): Result | null {
        if (visitor.visitIndirect) {
            return visitor.visitIndirect(this);
        } else {
            return visitor.visitChildren(this);
        }
    }
}


export class ImmediateContext extends antlr.ParserRuleContext {
    public constructor(parent: antlr.ParserRuleContext | null, invokingState: number) {
        super(parent, invokingState);
    }
    public EQUALS(): antlr.TerminalNode {
        return this.getToken(MemoryMachineAssemblyParser.EQUALS, 0)!;
    }
    public NUMBER(): antlr.TerminalNode {
        return this.getToken(MemoryMachineAssemblyParser.NUMBER, 0)!;
    }
    public override get ruleIndex(): number {
        return MemoryMachineAssemblyParser.RULE_immediate;
    }
    public override enterRule(listener: MemoryMachineAssemblyListener): void {
        if(listener.enterImmediate) {
             listener.enterImmediate(this);
        }
    }
    public override exitRule(listener: MemoryMachineAssemblyListener): void {
        if(listener.exitImmediate) {
             listener.exitImmediate(this);
        }
    }
    public override accept<Result>(visitor: MemoryMachineAssemblyVisitor<Result>): Result | null {
        if (visitor.visitImmediate) {
            return visitor.visitImmediate(this);
        } else {
            return visitor.visitChildren(this);
        }
    }
}


export class AccessorContext extends antlr.ParserRuleContext {
    public constructor(parent: antlr.ParserRuleContext | null, invokingState: number) {
        super(parent, invokingState);
    }
    public LBRACKET(): antlr.TerminalNode {
        return this.getToken(MemoryMachineAssemblyParser.LBRACKET, 0)!;
    }
    public index(): IndexContext {
        return this.getRuleContext(0, IndexContext)!;
    }
    public RBRACKET(): antlr.TerminalNode {
        return this.getToken(MemoryMachineAssemblyParser.RBRACKET, 0)!;
    }
    public override get ruleIndex(): number {
        return MemoryMachineAssemblyParser.RULE_accessor;
    }
    public override enterRule(listener: MemoryMachineAssemblyListener): void {
        if(listener.enterAccessor) {
             listener.enterAccessor(this);
        }
    }
    public override exitRule(listener: MemoryMachineAssemblyListener): void {
        if(listener.exitAccessor) {
             listener.exitAccessor(this);
        }
    }
    public override accept<Result>(visitor: MemoryMachineAssemblyVisitor<Result>): Result | null {
        if (visitor.visitAccessor) {
            return visitor.visitAccessor(this);
        } else {
            return visitor.visitChildren(this);
        }
    }
}


export class IndexContext extends antlr.ParserRuleContext {
    public constructor(parent: antlr.ParserRuleContext | null, invokingState: number) {
        super(parent, invokingState);
    }
    public direct(): DirectContext | null {
        return this.getRuleContext(0, DirectContext);
    }
    public indirect(): IndirectContext | null {
        return this.getRuleContext(0, IndirectContext);
    }
    public immediate(): ImmediateContext | null {
        return this.getRuleContext(0, ImmediateContext);
    }
    public override get ruleIndex(): number {
        return MemoryMachineAssemblyParser.RULE_index;
    }
    public override enterRule(listener: MemoryMachineAssemblyListener): void {
        if(listener.enterIndex) {
             listener.enterIndex(this);
        }
    }
    public override exitRule(listener: MemoryMachineAssemblyListener): void {
        if(listener.exitIndex) {
             listener.exitIndex(this);
        }
    }
    public override accept<Result>(visitor: MemoryMachineAssemblyVisitor<Result>): Result | null {
        if (visitor.visitIndex) {
            return visitor.visitIndex(this);
        } else {
            return visitor.visitChildren(this);
        }
    }
}

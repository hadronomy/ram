
import * as antlr from "antlr4ng";
import { Token } from "antlr4ng";


export class MemoryMachineAssemblyLexer extends antlr.Lexer {
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

    public static readonly channelNames = [
        "DEFAULT_TOKEN_CHANNEL", "HIDDEN"
    ];

    public static readonly literalNames = [
        null, null, null, "':'", "'='", "'*'", "'['", "']'"
    ];

    public static readonly symbolicNames = [
        null, "IDENTIFIER", "NUMBER", "COLON", "EQUALS", "ASTERISK", "LBRACKET", 
        "RBRACKET", "WS", "COMMENT", "NEWLINE"
    ];

    public static readonly modeNames = [
        "DEFAULT_MODE",
    ];

    public static readonly ruleNames = [
        "IDENTIFIER", "NUMBER", "COLON", "EQUALS", "ASTERISK", "LBRACKET", 
        "RBRACKET", "WS", "COMMENT", "NEWLINE",
    ];


    public constructor(input: antlr.CharStream) {
        super(input);
        this.interpreter = new antlr.LexerATNSimulator(this, MemoryMachineAssemblyLexer._ATN, MemoryMachineAssemblyLexer.decisionsToDFA, new antlr.PredictionContextCache());
    }

    public get grammarFileName(): string { return "MemoryMachineAssembly.g4"; }

    public get literalNames(): (string | null)[] { return MemoryMachineAssemblyLexer.literalNames; }
    public get symbolicNames(): (string | null)[] { return MemoryMachineAssemblyLexer.symbolicNames; }
    public get ruleNames(): string[] { return MemoryMachineAssemblyLexer.ruleNames; }

    public get serializedATN(): number[] { return MemoryMachineAssemblyLexer._serializedATN; }

    public get channelNames(): string[] { return MemoryMachineAssemblyLexer.channelNames; }

    public get modeNames(): string[] { return MemoryMachineAssemblyLexer.modeNames; }

    public static readonly _serializedATN: number[] = [
        4,0,10,59,6,-1,2,0,7,0,2,1,7,1,2,2,7,2,2,3,7,3,2,4,7,4,2,5,7,5,2,
        6,7,6,2,7,7,7,2,8,7,8,2,9,7,9,1,0,1,0,5,0,24,8,0,10,0,12,0,27,9,
        0,1,1,4,1,30,8,1,11,1,12,1,31,1,2,1,2,1,3,1,3,1,4,1,4,1,5,1,5,1,
        6,1,6,1,7,1,7,1,7,1,7,1,8,1,8,5,8,50,8,8,10,8,12,8,53,9,8,1,9,4,
        9,56,8,9,11,9,12,9,57,0,0,10,1,1,3,2,5,3,7,4,9,5,11,6,13,7,15,8,
        17,9,19,10,1,0,5,2,0,65,90,97,122,4,0,48,57,65,90,95,95,97,122,1,
        0,48,57,2,0,9,9,32,32,2,0,10,10,13,13,62,0,1,1,0,0,0,0,3,1,0,0,0,
        0,5,1,0,0,0,0,7,1,0,0,0,0,9,1,0,0,0,0,11,1,0,0,0,0,13,1,0,0,0,0,
        15,1,0,0,0,0,17,1,0,0,0,0,19,1,0,0,0,1,21,1,0,0,0,3,29,1,0,0,0,5,
        33,1,0,0,0,7,35,1,0,0,0,9,37,1,0,0,0,11,39,1,0,0,0,13,41,1,0,0,0,
        15,43,1,0,0,0,17,47,1,0,0,0,19,55,1,0,0,0,21,25,7,0,0,0,22,24,7,
        1,0,0,23,22,1,0,0,0,24,27,1,0,0,0,25,23,1,0,0,0,25,26,1,0,0,0,26,
        2,1,0,0,0,27,25,1,0,0,0,28,30,7,2,0,0,29,28,1,0,0,0,30,31,1,0,0,
        0,31,29,1,0,0,0,31,32,1,0,0,0,32,4,1,0,0,0,33,34,5,58,0,0,34,6,1,
        0,0,0,35,36,5,61,0,0,36,8,1,0,0,0,37,38,5,42,0,0,38,10,1,0,0,0,39,
        40,5,91,0,0,40,12,1,0,0,0,41,42,5,93,0,0,42,14,1,0,0,0,43,44,7,3,
        0,0,44,45,1,0,0,0,45,46,6,7,0,0,46,16,1,0,0,0,47,51,5,35,0,0,48,
        50,8,4,0,0,49,48,1,0,0,0,50,53,1,0,0,0,51,49,1,0,0,0,51,52,1,0,0,
        0,52,18,1,0,0,0,53,51,1,0,0,0,54,56,7,4,0,0,55,54,1,0,0,0,56,57,
        1,0,0,0,57,55,1,0,0,0,57,58,1,0,0,0,58,20,1,0,0,0,5,0,25,31,51,57,
        1,6,0,0
    ];

    private static __ATN: antlr.ATN;
    public static get _ATN(): antlr.ATN {
        if (!MemoryMachineAssemblyLexer.__ATN) {
            MemoryMachineAssemblyLexer.__ATN = new antlr.ATNDeserializer().deserialize(MemoryMachineAssemblyLexer._serializedATN);
        }

        return MemoryMachineAssemblyLexer.__ATN;
    }


    private static readonly vocabulary = new antlr.Vocabulary(MemoryMachineAssemblyLexer.literalNames, MemoryMachineAssemblyLexer.symbolicNames, []);

    public override get vocabulary(): antlr.Vocabulary {
        return MemoryMachineAssemblyLexer.vocabulary;
    }

    private static readonly decisionsToDFA = MemoryMachineAssemblyLexer._ATN.decisionToState.map( (ds: antlr.DecisionState, index: number) => new antlr.DFA(ds, index) );
}
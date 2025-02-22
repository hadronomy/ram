import type monaco from 'monaco-editor';
import { BaseErrorListener, Token } from 'antlr4ng';

import { createLexer } from './utils';

type ILineTokens = monaco.languages.ILineTokens;
type IToken = monaco.languages.IToken;
type IState = monaco.languages.IState;

const GRAMMAR_CONSTANTS = {
    EOF: -1,
    ERROR_SCOPE: 'error.ram',
    FILE_EXTENSION: '.ram'
} as const;

export class RAMState implements IState {
    public clone(): IState {
        return new RAMState();
    }

    public equals(other: IState | null): boolean {
        return other instanceof RAMState;
    }
}

export class RAMToken implements IToken {
    public readonly scopes: string;
    public readonly startIndex: number;

    constructor(ruleName: string, startIndex: number) {
        this.scopes = `${ruleName.toLowerCase()}${GRAMMAR_CONSTANTS.FILE_EXTENSION}`;
        this.startIndex = startIndex;
    }
}

class RAMLineTokens implements ILineTokens {
    public readonly tokens: IToken[];
    public readonly endState: RAMState;

    constructor(tokens: readonly IToken[], endState: RAMState) {
        // Convert readonly array to mutable array to match interface
        this.tokens = [...tokens];
        this.endState = endState;
    }
}

class ErrorCollectorListener extends BaseErrorListener {
    private readonly errors: number[] = [];

    public syntaxError(_recognizer: unknown, _offendingSymbol: unknown, 
        // eslint-disable-next-line @typescript-eslint/no-unused-vars
        line: number, charPositionInLine: number, _msg: string, _e: unknown): void {
        this.errors.push(line * charPositionInLine);
    }

    public getErrors(): readonly number[] {
        return [...this.errors];
    }
}

export class RAMTokenProvider implements monaco.languages.TokensProvider {
    public getInitialState(): IState {
        return new RAMState();
    }

    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    public tokenize(line: string, _state: IState): ILineTokens {
        return lineTokens(line);
    }
}

export function lineTokens(input: string): ILineTokens {
    const errorListener = new ErrorCollectorListener();
    const lexer = createLexer(input);
    
    // Configure lexer
    lexer.removeErrorListeners();
    lexer.addErrorListener(errorListener);

    // Process tokens
    const tokens: IToken[] = [];
    let token: Token | null;
    
    while ((token = lexer.nextToken()) !== null) {
        if (token.type === GRAMMAR_CONSTANTS.EOF) {
            break;
        }

        const tokenTypeName = lexer.symbolicNames[token.type];
        tokens.push(new RAMToken(tokenTypeName, token.column));
    }

    // Add error tokens
    const errorTokens = errorListener.getErrors().map(
        pos => new RAMToken(GRAMMAR_CONSTANTS.ERROR_SCOPE, pos)
    );

    // Combine and sort all tokens
    const allTokens = [...tokens, ...errorTokens].sort(
        (a, b) => a.startIndex - b.startIndex
    );

    return new RAMLineTokens(allTokens, new RAMState());
}
import type { Token } from 'antlr4ng';
import type * as monaco from 'monaco-editor';
import { BaseErrorListener } from 'antlr4ng';

import { createLexer } from './utils';

type ILineTokens = monaco.languages.ILineTokens;
type IToken = monaco.languages.IToken;
type IState = monaco.languages.IState;

const GRAMMAR_CONSTANTS = {
  EOF: -1,
  ERROR_SCOPE: 'error.ram',
  FILE_EXTENSION: '.ram',
} as const;

const TOKEN_MAP: Record<string, string> = {
  IDENTIFIER: 'entity.name.function',
  NUMBER: 'constant.numeric',
  COLON: 'delimiter',
  EQUALS: 'operator',
  ASTERISK: 'operator',
  LBRACKET: 'punctuation.bracket',
  RBRACKET: 'punctuation.bracket',
  COMMENT: 'comment',
  NEWLINE: 'whitespace',
  [GRAMMAR_CONSTANTS.ERROR_SCOPE]: 'error',
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
    this.scopes = TOKEN_MAP[ruleName] || `${ruleName.toLowerCase()}${GRAMMAR_CONSTANTS.FILE_EXTENSION}`;
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

  public syntaxError(_recognizer: unknown, _offendingSymbol: unknown, line: number, charPositionInLine: number, _msg: string, _e: unknown): void {
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
  let token: Token | null = lexer.nextToken();

  while (token !== null) {
    if (token.type === GRAMMAR_CONSTANTS.EOF) {
      break;
    }

    const tokenTypeName = lexer.symbolicNames[token.type];
    tokens.push(new RAMToken(tokenTypeName, token.column));
    token = lexer.nextToken();
  }

  // Add error tokens
  const errorTokens = errorListener.getErrors().map(
    pos => new RAMToken(GRAMMAR_CONSTANTS.ERROR_SCOPE, pos),
  );

  // Combine and sort all tokens
  const allTokens = [...tokens, ...errorTokens].sort(
    (a, b) => a.startIndex - b.startIndex,
  );

  return new RAMLineTokens(allTokens, new RAMState());
}

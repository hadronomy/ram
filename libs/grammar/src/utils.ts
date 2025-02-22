import { CharStream, CommonTokenStream, Token } from 'antlr4ng';

import { MemoryMachineAssemblyLexer, MemoryMachineAssemblyParser } from './generated';

export function createLexer(input: string): MemoryMachineAssemblyLexer {
    const chars = CharStream.fromString(input);
    const lexer = new MemoryMachineAssemblyLexer(chars);
    return lexer;
}

export function getTokens(input: string) : Token[] {
    return createLexer(input).getAllTokens()
}

export function createParser(input: string) {
    const lexer = createLexer(input);
    return createParserFromLexer(lexer);
}

export function createParserFromLexer(lexer: MemoryMachineAssemblyLexer) {
    const tokens = new CommonTokenStream(lexer);
    return new MemoryMachineAssemblyParser(tokens);
}

export function parse(input: string) {
    const parser = createParser(input);
    return parser.program();
}

export function parseIntoStr(input: string) {
    return parse(input).toStringTree();
}

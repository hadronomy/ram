grammar MemoryMachineAssembly;

@header {#pragma warning disable 3021}

// Parser Rules

program: line (NEWLINE+ line)* NEWLINE* EOF;

line: instruction comment? | label_definition | comment;
label_definition:
	IDENTIFIER COLON NEWLINE? instruction comment?;
label: IDENTIFIER;

instruction: IDENTIFIER argument?;

argument:
	direct		# NumberOperand
	| indirect	# IndirectOperand
	| immediate	# ImmediateOperand
	| label		# LabelOperand;

comment: COMMENT;

direct: NUMBER accessor?; // e.g 1 o 1[5]
indirect: ASTERISK NUMBER;
immediate: EQUALS NUMBER;

accessor: LBRACKET index RBRACKET;
index: direct | indirect | immediate;

// Lexer Rules
IDENTIFIER: [a-zA-Z] [a-zA-Z0-9_]*;

NUMBER: [0-9]+;

COLON: ':';
EQUALS: '=';
ASTERISK: '*';
LBRACKET: '[';
RBRACKET: ']';

WS: [ \t] -> skip;
COMMENT: '#' ~[\r\n]*;

NEWLINE: [\r\n]+;

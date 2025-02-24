use chumsky::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

// --- Data Structures ---

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Program {
    lines: Vec<Line>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum Line {
    Instruction {
        instruction: Instruction,
        comment: Option<String>,
    },
    LabelDefinition {
        label: String,
        instruction: Instruction,
        comment: Option<String>,
    },
    Comment(String),
    Invalid,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum Instruction {
    Basic {
        opcode: String,
        arg: Option<Operand>,
    },
    Invalid,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum Operand {
    Number(NumberOperand),
    Indirect(IndirectOperand),
    Immediate(ImmediateOperand),
    Label(String),
    Invalid,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct NumberOperand {
    value: u64,
    accessor: Option<Accessor>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct IndirectOperand {
    value: u64,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ImmediateOperand {
    value: u64,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Accessor {
    index: Box<Operand>,
}

// --- Parser ---

pub fn parser() -> impl Parser<char, Program, Error = Simple<char>> {
    program_parser()
}

fn program_parser() -> impl Parser<char, Program, Error = Simple<char>> {
    line_parser()
        .padded()
        .separated_by(newline_parser().or(inline_whitespace_parser()))
        .allow_trailing()
        .map(|lines| Program {
            lines: lines
                .into_iter()
                .filter(|line| !matches!(line, Line::Invalid))
                .collect(),
        })
        .recover_with(skip_parser(end().map(|_| Program { lines: vec![] })))
        .labelled("program")
}

fn line_parser() -> impl Parser<char, Line, Error = Simple<char>> {
    choice((
        label_definition_parser(),
        instruction_parser().map(|inst| Line::Instruction {
            instruction: inst,
            comment: None,
        }),
        comment_parser().map(Line::Comment),
    ))
    .padded()
    .recover_with(skip_then_retry_until(['\n']))
    .labelled("line")
}

fn label_definition_parser() -> impl Parser<char, Line, Error = Simple<char>> {
    identifier_parser()
        .then_ignore(just(':').padded())
        .then(instruction_parser().padded())
        .then(comment_parser().or_not())
        .map(|((label, instruction), comment)| Line::LabelDefinition {
            label,
            instruction,
            comment,
        })
        .labelled("label definition")
}

fn instruction_parser() -> impl Parser<char, Instruction, Error = Simple<char>> {
    identifier_parser()
        .padded()
        .then(operand_parser().padded().or_not())
        .map(|(opcode, arg)| Instruction::Basic { opcode, arg })
        .labelled("instruction")
}

fn operand_parser() -> impl Parser<char, Operand, Error = Simple<char>> {
    recursive(|operand| {
        let accessor = operand
            .clone()
            .padded()
            .delimited_by(
                just('[').labelled("opening bracket"),
                just(']')
                    .ignored()
                    .recover_with(skip_then_retry_until(['\n', ' ', '#']))
                    .labelled("closing bracket"),
            )
            .map(|idx| Accessor {
                index: Box::new(idx),
            })
            .labelled("array accessor");

        let direct = number_parser()
            .then(accessor.or_not())
            .map(|(value, acc)| Operand::Number(NumberOperand { value, accessor: acc }))
            .labelled("direct operand");

        let indirect = just('*')
            .padded()
            .ignore_then(number_parser())
            .map(|value| Operand::Indirect(IndirectOperand { value }))
            .labelled("indirect operand");

        let immediate = just('=')
            .padded()
            .ignore_then(number_parser())
            .map(|value| Operand::Immediate(ImmediateOperand { value }))
            .labelled("immediate operand");

        let label_operand = identifier_parser().map(Operand::Label).labelled("label operand");

        direct
            .or(immediate)
            .or(indirect)
            .or(label_operand)
            .recover_with(skip_then_retry_until(['\n', ' ', '#']))
            .labelled("operand")
    })
}

fn comment_parser() -> impl Parser<char, String, Error = Simple<char>> {
    just('#')
        .then(take_until(newline_parser().or(end())))
        .map(|(_, (chars, _))| chars.into_iter().collect::<String>().trim().to_string())
        .labelled("comment")
}

fn identifier_parser() -> impl Parser<char, String, Error = Simple<char>> {
    filter(|c: &char| c.is_alphabetic())
        .chain(filter(|c: &char| c.is_alphanumeric() || *c == '_').repeated())
        .collect::<String>()
        .labelled("identifier")
}

fn number_parser() -> impl Parser<char, u64, Error = Simple<char>> {
    text::int(10)
        .map(|s: String| s.parse().unwrap()) // TODO: Handle parse errors properly
        .labelled("number")
}

fn newline_parser() -> impl Parser<char, (), Error = Simple<char>> {
    text::newline().repeated().at_least(1).ignored().labelled("newline")
}

fn inline_whitespace_parser() -> impl Parser<char, (), Error = Simple<char>> {
    filter(|c: &char| c.is_whitespace()).repeated().ignored()
}

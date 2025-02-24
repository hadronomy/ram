use chumsky::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

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
    Instruction(Instruction, Option<String>), // instruction with optional comment
    LabelDefinition {
        label: String,
        instruction: Instruction,
        comment: Option<String>,
    },
    Comment(String),
    Invalid,
}

pub fn parser() -> impl Parser<char, Program, Error = Simple<char>> {
    // Basic parsers
    let newline = text::newline().repeated().at_least(1).ignored().labelled("newline");

    // Modified inline_whitespace to be more lenient
    let inline_whitespace = filter(|c: &char| c.is_whitespace()).repeated().ignored();

    let identifier = filter(|c: &char| c.is_alphabetic())
        .chain(filter(|c: &char| c.is_alphanumeric() || *c == '_').repeated())
        .collect::<String>()
        .labelled("identifier");

    let number = text::int(10)
        .map(|s: String| s.parse().unwrap())
        .labelled("number");

    // Comment parser
    let comment = just('#')
        .then(take_until(newline.or(end())))
        .map(|(_, (chars, _))| {
            chars.into_iter().collect::<String>().trim().to_string()
        })
        .labelled("comment");

    // Operand parsers
    let operand = recursive(|operand| {
        let accessor = recursive(|_accessor| {
            operand
            .clone()
            .padded_by(inline_whitespace)
            .delimited_by(
                just('[').labelled("opening bracket"),
                just(']')
                    .ignored()
                    .recover_with(skip_then_retry_until(['\n', ' ', '#']))
                    .labelled("closing bracket")
            )
            .map(|idx| Accessor {
                index: Box::new(idx),
            })
            .labelled("array accessor")
        });

        let direct = number
            .then(
                accessor.or_not()
            )
            .map(|(value, acc)| Operand::Number(NumberOperand { value, accessor: acc }))
            .labelled("direct operand");

        let indirect = just('*')
            .padded_by(inline_whitespace)
            .ignore_then(number)
            .map(|value| Operand::Indirect(IndirectOperand { value }))
            .labelled("indirect operand");

        let immediate = just('=')
            .padded_by(inline_whitespace)
            .ignore_then(number)
            .map(|value| Operand::Immediate(ImmediateOperand { value }))
            .labelled("immediate operand");

        let label_operand = identifier.map(Operand::Label).labelled("label operand");

        // Try parsing direct numbers with array accessors first
        direct
            .or(immediate)
            .or(indirect)
            .or(label_operand)
            .recover_with(skip_then_retry_until(['\n', ' ', '#']))
            .labelled("operand")
    });

    let instruction = identifier
        .padded_by(inline_whitespace)
        .then(operand.padded_by(inline_whitespace).or_not())
        .map(|(opcode, arg)| Instruction::Basic { opcode, arg })
        .labelled("instruction");

    let label_def = identifier
        .then_ignore(just(':').padded_by(inline_whitespace))
        .then(instruction.clone().padded_by(inline_whitespace))
        .then(comment.clone().or_not())
        .map(|((label, instruction), comment)| Line::LabelDefinition {
            label,
            instruction,
            comment,
        })
        .labelled("label definition");

        let line = choice((
            label_def,
            instruction
                .then(comment.clone().or_not())
                .map(|(inst, comment)| Line::Instruction(inst, comment)),
            comment.clone().map(Line::Comment)
        ))
        .padded_by(inline_whitespace)
        .recover_with(skip_then_retry_until(['\n']))
        .labelled("line");

    let program = line
        .padded_by(inline_whitespace)
        .separated_by(newline.or(inline_whitespace))
        .allow_trailing()
        .map(|lines| Program {
            lines: lines
                .into_iter()
                .filter(|line| !matches!(line, Line::Invalid))
                .collect(),
        })
        .recover_with(skip_parser(end().map(|_| Program { lines: vec![] })))
        .labelled("program");

    #[expect(clippy::let_and_return)]
    program
}

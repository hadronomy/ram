use std::ffi::OsString;
use std::process::ExitCode;

use chumsky::{stream, Parser as ChumskyParser};
use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use clap::{CommandFactory, Parser};
use miette::*;

use ram_cli::{Cli, Command};
pub use error::Error;

pub mod error;
pub mod lang;

pub fn main<Args, T>(args: Args) -> Result<ExitCode>
where
    Args: Iterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(err) => {
            err.exit();
        }
    };

    miette::set_hook(Box::new(|_| {
        Box::new(
            miette::MietteHandlerOpts::new()
                .break_words(false)
                .word_separator(textwrap::WordSeparator::AsciiSpace)
                .word_splitter(textwrap::WordSplitter::NoHyphenation)
                .build(),
        )
    }))
    .map_err(|err| Error::SetupError(err.into()))?;

    match *cli.command {
        // execute help
        Command::Help(_) => {
            Cli::command().print_help().into_diagnostic()?;
            Ok::<_, Error>(ExitCode::SUCCESS)
        },
        Command::Validate => {
            // execute validate
            let src = r#"# Bubble Sort - Initial setup
        read 0[2[]      # Read size into R0
        store 10    # Store size in R10 (permanent)
        load =0
        store 2     # Initialize array index to 0

# Read array elements
read_loop: load 10
        sub 2
        jzero end_read
        read 0      # Read next element
        store 3[2]  # Store in array
        load 2
        add =1
        store 2     # Increment index
        jump read_loop

# Initialize sorting
end_read: load 10
        sub =1
        store 1     # n-1 in R1 (outer loop counter)

outer:  load 1      # Check if outer loop done
        jzero end_outer
        load =0
        store 2     # j = 0 (inner loop counter)

inner:  load 1
        sub 2       # Check if inner loop done
        jzero next_outer
        load 3[2]   # Load current element
        store 4     # Store in R4
        load 2
        add =1
        store 5     # Index for next element
        load 3[5]   # Load next element
        sub 4       # Compare next - current
        jgtz next_inner  # If next > current, no swap needed

# Swap elements
        load 3[5]   # Load next element
        store 6     # Store temporarily
        load 4      # Load current element
        store 3[5]  # Store current in next position
        load 6      # Load saved next element
        store 3[2]  # Store in current position

next_inner:
        load 2
        add =1
        store 2     # j++
        jump inner

next_outer:
        load 1
        sub =1
        store 1     # Decrement outer loop counter
        jump outer

# Print sorted array
end_outer: load =0
        store 2     # Reset counter
print:  load 10
        sub 2
        jzero terminate
        load 3[2]
        write 0
        load 2
        add =1
        store 2
        jump print

terminate: halt
            "#;
            let (program, errors) = lang::parser().parse_recovery(stream::Stream::from(src));
            errors.into_iter().for_each(|e| {
                let msg = if let chumsky::error::SimpleReason::Custom(msg) = e.reason() {
                    msg.clone()
                } else {
                    format!(
                        "{}{}, expected {}",
                        if e.found().is_some() {
                            "Unexpected token"
                        } else {
                            "Unexpected end of input"
                        },
                        if let Some(label) = e.label() {
                            format!(" while parsing {}", label)
                        } else {
                            String::new()
                        },
                        if e.expected().len() == 0 {
                            "something else".to_string()
                        } else {
                            e.expected()
                                .map(|expected| match expected {
                                    Some(expected) => expected.to_string(),
                                    None => "end of input".to_string(),
                                })
                                .collect::<Vec<_>>()
                                .join(", ")
                        },
                    )
                };

                let report = Report::build(ReportKind::Error, e.span())
                    .with_code(3)
                    .with_message(msg)
                    .with_label(
                        Label::new(e.span())
                            .with_message(match e.reason() {
                                chumsky::error::SimpleReason::Custom(msg) => msg.clone(),
                                _ => format!(
                                    "Unexpected {}",
                                    e.found()
                                        .map(|c| format!("token {}", c.fg(Color::Red)))
                                        .unwrap_or_else(|| "end of input".to_string())
                                ),
                            })
                            .with_color(Color::Red),
                    );

                let report = match e.reason() {
                    chumsky::error::SimpleReason::Unclosed { span, delimiter } => report.with_label(
                        Label::new(span.clone())
                            .with_message(format!(
                                "Unclosed delimiter {}",
                                delimiter.fg(Color::Yellow)
                            ))
                            .with_color(Color::Yellow),
                    ),
                    chumsky::error::SimpleReason::Unexpected => report,
                    chumsky::error::SimpleReason::Custom(_) => report,
                };

                report.finish().print(Source::from(&src)).unwrap();
            });


            #[cfg(feature = "serde")]
            println!("{}", serde_json::to_string_pretty(&program).unwrap());
            // print normaly of not serde
            #[cfg(not(feature = "serde"))]
            println!("{:#?}", program);
            Ok::<_, Error>(ExitCode::SUCCESS)
        },
        _ => Err(Error::Unimplemented),
    }
    .wrap_err("Failed to execute command")
}

using Antlr4.Runtime;
using System;
using System.Text;
using System.IO;
using Antlr4.Runtime.Misc;

namespace MemoryMachine.Core.Parser
{
    public class AssemblyErrorListener : BaseErrorListener
    {
        // ANSI color codes
        private const string Red = "\u001b[31m";
        private const string Blue = "\u001b[34m";
        private const string Reset = "\u001b[0m";

        public bool HasError { get; private set; } = false;

        public override void SyntaxError(
            TextWriter output,
            IRecognizer recognizer,
            IToken? offendingSymbol,
            int line,
            int charPositionInLine,
            string msg,
            RecognitionException e
        )
        {
            HasError = true;
            StringBuilder errorMessage = new StringBuilder();

            // First line: error header
            errorMessage.AppendLine($"{Red}error:{Reset} {msg}");

            // Second line: location information
            errorMessage.AppendLine($"{Blue}  --> {Reset}line {line}, char {charPositionInLine}");

            // Get the input stream based on the type of recognizer
            ICharStream? inputStream = null;
            string? offendingText = null;

            if (offendingSymbol is not null && recognizer is Antlr4.Runtime.Parser)
            {
                if (offendingSymbol.TokenSource?.InputStream is ICharStream stream)
                {
                    inputStream = stream;
                    offendingText = offendingSymbol.Text;
                }
            }
            else if (recognizer is Lexer lexer && lexer.InputStream is ICharStream stream)
            {
                inputStream = stream;
                offendingText = inputStream.GetText(
                    new Interval(charPositionInLine, charPositionInLine)
                );
            }

            if (inputStream is not null)
            {
                string? lineContent = inputStream.ToString();
                if (lineContent is not null)
                {
                    string[] lines = lineContent.Split('\n');

                    if (line - 1 < lines.Length)
                    {
                        // Show the line content
                        string errorLine = lines[line - 1].TrimEnd('\r').Replace("\t", "    ");
                        errorMessage.AppendLine($"{Blue}{line,3} |{Reset} {errorLine}");

                        // Calculate the actual position including leading whitespace
                        int actualPosition = charPositionInLine;
                        string normalizedLine = errorLine;
                        for (int i = 0; i < charPositionInLine && i < normalizedLine.Length; i++)
                        {
                            if (normalizedLine[i] == ' ')
                            {
                                actualPosition++;
                            }
                        }

                        // Show the error pointer with correct spacing
                        errorMessage.Append($"{Blue}    |{Reset} ");
                        errorMessage.Append(new string(' ', actualPosition));

                        // Determine the length of the error indicator
                        int errorLength = offendingText?.Length ?? 1;
                        errorMessage.AppendLine($"{Red}{new string('^', errorLength)}{Reset}");

                        // Add help message
                        string helpMessage = offendingText is not null
                            ? $"help: unexpected token '{offendingText}'"
                            : "help: invalid character";

                        errorMessage.AppendLine(
                            $"{Blue}    |{Reset} {new string(' ', actualPosition)}{Blue} {helpMessage}{Reset}"
                        );
                    }
                }
            }

            output.Write(errorMessage.ToString());
        }
    }
}

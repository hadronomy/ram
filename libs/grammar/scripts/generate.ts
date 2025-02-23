import { execSync } from 'node:child_process';
import * as fs from 'node:fs';
import * as path from 'node:path';
import * as process from 'node:process';
import { fileURLToPath } from 'node:url';
import chalk from 'chalk';
import * as tmp from 'tmp';

// Get the directory name in ESM
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

function findProjectRoot(): string | null {
  let currentDir = __dirname;
  while (currentDir !== path.dirname(currentDir)) {
    if (fs.existsSync(path.join(currentDir, 'pnpm-workspace.yaml'))) {
      return currentDir;
    }
    currentDir = path.dirname(currentDir);
  }
  return null;
}

// Get the project root directory
const projectRoot = findProjectRoot();

if (!projectRoot) {
  console.error(chalk.red('Error: Could not find project root (pnpm-workspace.yaml not found).'));
  process.exit(1);
}

interface Config {
  grammarFile: string;
  outputDirTS: string;
  outputDirCS: string;
  cSharpNamespace: string;
  noListener: boolean;
  noVisitor: boolean;
}

const config: Config = {
  grammarFile: path.resolve(projectRoot, 'libs/grammar/src/MemoryMachineAssembly.g4'),
  outputDirTS: path.resolve(projectRoot, 'libs/grammar/src/generated/ts'),
  outputDirCS: path.resolve(projectRoot, 'libs/grammar/src/generated/cs'),
  cSharpNamespace: 'MemoryMachine.Parser',
  noListener: false,
  noVisitor: false,
};

function cleanDirectory(dir: string): void {
  if (fs.existsSync(dir)) {
    console.log(chalk.blue(`Cleaning directory: ${dir}`));
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

function ensureDirectoryExistence(filePath: string): void {
  const dirname = path.dirname(filePath);
  if (fs.existsSync(dirname)) {
    return;
  }
  ensureDirectoryExistence(dirname);
  fs.mkdirSync(dirname);
}

function createTempGrammarFile(grammarFile: string): {
  tempFile: string;
  cleanup: () => void;
} {
  const grammarContent = fs.readFileSync(grammarFile, 'utf8');
  const originalFileName = path.basename(grammarFile);
  const tmpDir = tmp.dirSync({ unsafeCleanup: true });
  const tempFile = path.join(tmpDir.name, originalFileName);

  const contentWithPragma = grammarContent.split('\n')
    .reduce((acc, line, index) => {
      if (index === 0) {
        return `${acc}${line}\n@header {#pragma warning disable 3021}\n`;
      }
      return `${acc}${line}\n`;
    }, '');

  fs.writeFileSync(tempFile, contentWithPragma);
  return {
    tempFile,
    cleanup: () => tmpDir.removeCallback(),
  };
}

function buildAntlrCommand(
  language: 'TypeScript' | 'CSharp',
  outputDir: string,
  grammarFile: string,
  namespace: string | null,
  noListener: boolean,
  noVisitor: boolean,
): string {
  const baseCommand = language === 'TypeScript'
    ? `antlr-ng -Dlanguage=TypeScript -o ${outputDir} ${grammarFile}`
    : `antlr-ng -Dlanguage=CSharp -o ${outputDir} ${grammarFile}`;

  return [
    baseCommand,
    !noListener && '-l',
    !noVisitor && '-v',
  ].filter(Boolean).join(' ');
}

function generateAntlr(
  grammarFile: string,
  outputDir: string,
  language: 'TypeScript' | 'CSharp',
  namespace: string | null,
  noListener: boolean,
  noVisitor: boolean,
): void {
  if (!fs.existsSync(grammarFile)) {
    console.error(chalk.red(`Error: Grammar file not found: ${grammarFile}`));
    process.exit(1);
  }

  // Clean and ensure output directory exists
  cleanDirectory(outputDir);
  ensureDirectoryExistence(outputDir);

  let cleanup: (() => void) | null = null;
  let effectiveGrammarFile = grammarFile;

  try {
    // Handle C# specific requirements
    if (language === 'CSharp') {
      const temp = createTempGrammarFile(grammarFile);
      effectiveGrammarFile = temp.tempFile;
      cleanup = temp.cleanup;
    }

    const command = buildAntlrCommand(
      language,
      outputDir,
      effectiveGrammarFile,
      namespace,
      noListener,
      noVisitor,
    );

    console.log(chalk.blue(`Generating ${language} parser...`));
    console.log(chalk.gray(`Executing: ${command}`));

    execSync(command, { stdio: 'inherit' });

    console.log(chalk.green(`Successfully generated ${language} parser in ${outputDir}`));
  }
  // eslint-disable-next-line unused-imports/no-unused-vars
  catch (error) {
    console.error(chalk.red(`ANTLR generation failed for ${language}.`));
    if (cleanup)
      cleanup();
    process.exit(1);
  }

  if (cleanup)
    cleanup();
}

function main(): void {
  console.log(chalk.bold.underline('ANTLR Parser Generator'));

  try {
    generateAntlr(
      config.grammarFile,
      config.outputDirTS,
      'TypeScript',
      null,
      config.noListener,
      config.noVisitor,
    );

    generateAntlr(
      config.grammarFile,
      config.outputDirCS,
      'CSharp',
      config.cSharpNamespace,
      config.noListener,
      config.noVisitor,
    );

    console.log(chalk.green.bold('All parsers generated successfully!'));
  }
  // eslint-disable-next-line unused-imports/no-unused-vars
  catch (error) {
    console.error(chalk.red.bold('Parser generation failed. See errors above.'));
    process.exit(1);
  }
}

main();

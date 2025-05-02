import * as vscode from 'vscode';

// Registry decoration types (dynamically created for registers)
const registerDecorations = new Map<number, vscode.TextEditorDecorationType>();

/**
 * Generates a pastel color for a register based on its number
 * Uses the golden ratio to distribute colors evenly around the color wheel
 */
function generateRegisterColor(registerNum: number): { text: string } {
  // Use the golden ratio to create aesthetic color distribution
  const goldenRatioConjugate = 0.618033988749895;
  const hue = (registerNum * goldenRatioConjugate * 360) % 360;

  // Convert to HSL color - pastel colors have high lightness and medium saturation
  const h = hue.toFixed(0);
  const s = 70;
  const l = 70;

  return {
    text: `hsla(${h}, ${s}%, ${l}%, 1)`,
  };
}

/**
 * Creates a text editor decoration type for a specific register
 */
function createRegisterDecoration(registerNum: number): vscode.TextEditorDecorationType {
  const colors = generateRegisterColor(registerNum);

  // Simplified decoration: only color and font weight
  return vscode.window.createTextEditorDecorationType({
    color: colors.text,
    fontWeight: 'bold',
    rangeBehavior: vscode.DecorationRangeBehavior.ClosedClosed,
  });
}

/**
 * Gets or creates a decoration for a specific register
 */
function getRegisterDecoration(registerNum: number): vscode.TextEditorDecorationType {
  if (!registerDecorations.has(registerNum)) {
    registerDecorations.set(registerNum, createRegisterDecoration(registerNum));
  }
  return registerDecorations.get(registerNum)!;
}

/**
 * Initialize decorations for the extension
 */
export function initDecorations(context: vscode.ExtensionContext) {
  // Removed creation of non-register decoration types

  // Register the toggle command
  context.subscriptions.push(
    vscode.commands.registerCommand('ram.toggleDecorations', toggleDecorations),
  );

  // Set up event listeners
  registerEventListeners(context);
}

/**
 * Toggle decoration visibility
 */
async function toggleDecorations() {
  const config = vscode.workspace.getConfiguration('ram');
  const currentValue = config.get<boolean>('decorations.enabled', true);

  // Toggle the value
  await config.update('decorations.enabled', !currentValue, vscode.ConfigurationTarget.Global);

  vscode.window.showInformationMessage(
    `RAM register decorations ${!currentValue ? 'enabled' : 'disabled'}`,
  );

  // Update decorations immediately
  if (vscode.window.activeTextEditor) {
    updateDecorations(vscode.window.activeTextEditor);
  }
}

/**
 * Register event listeners for decoration updates
 */
function registerEventListeners(context: vscode.ExtensionContext) {
  let activeEditor = vscode.window.activeTextEditor;

  // Update when text changes
  context.subscriptions.push(
    vscode.workspace.onDidChangeTextDocument((event) => {
      if (activeEditor && event.document === activeEditor.document) {
        updateDecorations(activeEditor);
      }
    }),
  );

  // Update when active editor changes
  context.subscriptions.push(
    vscode.window.onDidChangeActiveTextEditor((editor) => {
      activeEditor = editor;
      if (editor) {
        updateDecorations(editor);
      }
    }),
  );

  // Update when configuration changes
  context.subscriptions.push(
    vscode.workspace.onDidChangeConfiguration((e) => {
      if (e.affectsConfiguration('ram.decorations.enabled') && activeEditor) {
        updateDecorations(activeEditor);
      }
    }),
  );

  // Initial update for the active editor
  if (activeEditor) {
    updateDecorations(activeEditor);
  }
}

// Update decorations in the editor
function updateDecorations(editor: vscode.TextEditor | undefined) {
  if (!editor || editor.document.languageId !== 'ram') {
    return;
  }

  // Check if decorations are enabled in settings
  const config = vscode.workspace.getConfiguration('ram');
  const decorationsEnabled = config.get<boolean>('decorations.enabled', true);
  if (!decorationsEnabled) {
    // Clear decorations if disabled
    clearDecorations(editor);
    return;
  }

  const text = editor.document.getText();
  const decorations = collectDecorations(editor, text);

  // Apply decorations
  applyDecorations(editor, decorations);
}

/**
 * Types to store decoration data - Only registers now
 */
interface DecorationCollections {
  registers: Map<number, vscode.DecorationOptions[]>;
}

/**
 * Collect all decorations from the text
 */
function collectDecorations(editor: vscode.TextEditor, text: string): DecorationCollections {
  // Simplified collections
  const collections: DecorationCollections = {
    registers: new Map(),
  };

  // Find registers associated with operators (but don't decorate operators)
  collectAssociatedRegisters(editor, text, collections);

  // Find standalone register numbers (not part of operators)
  collectStandaloneRegisters(editor, text, collections);

  return collections;
}

/**
 * Collect registers associated with operators (* or =)
 */
function collectAssociatedRegisters(
  editor: vscode.TextEditor,
  text: string,
  collections: DecorationCollections,
) {
  // Match only * or = that might be followed by a number
  const operatorRegex = /[*=]/g;
  let match: RegExpExecArray | null;

  // eslint-disable-next-line no-cond-assign
  while ((match = operatorRegex.exec(text)) !== null) {
    // Check if followed by a digit
    const afterMatchIndex = match.index + 1;
    if (afterMatchIndex < text.length && /\d/.test(text.charAt(afterMatchIndex))) {
      // Extract the full number (can be multiple digits)
      const numRegex = /\d+/;
      const afterText = text.substring(afterMatchIndex);
      const numMatch = afterText.match(numRegex);

      if (numMatch && numMatch.index === 0) {
        const registerNumStr = numMatch[0];
        const registerNum = Number.parseInt(registerNumStr, 10);

        // Only create decoration for the register number
        addRegisterDecoration(editor, afterMatchIndex, registerNumStr.length, registerNum, collections);
        // Skip decorating the operator itself
      }
    }
    // Don't decorate the operator if it's not followed by a number
  }
}

/**
 * Add a register decoration to the collections
 */
function addRegisterDecoration(
  editor: vscode.TextEditor,
  index: number,
  length: number,
  registerNum: number,
  collections: DecorationCollections,
) {
  const registerStartPos = editor.document.positionAt(index);
  const registerEndPos = editor.document.positionAt(index + length);
  const registerRange = new vscode.Range(registerStartPos, registerEndPos);

  // Simplified decoration options - only range and hover message
  const decoration: vscode.DecorationOptions = {
    range: registerRange,
    hoverMessage: new vscode.MarkdownString(`**Register ${registerNum}**`),
  };

  if (!collections.registers.has(registerNum)) {
    collections.registers.set(registerNum, []);
  }

  collections.registers.get(registerNum)!.push(decoration);
}

/**
 * Collect standalone registers from the text
 * - Recognizes numbers in instructions (e.g., `LOAD 123`)
 * - Recognizes numbers in comments ONLY if enclosed in backticks (e.g., `# `123``)
 */
function collectStandaloneRegisters(
  editor: vscode.TextEditor,
  text: string, // Full text needed for context checks (e.g., preceding char)
  collections: DecorationCollections,
) {
  const backtickRegisterRegex = /`(\d+)`/g; // Regex for backticked numbers
  const instructionRegisterRegex = /\b(\d+)\b/g; // Regex for standalone numbers in instructions

  for (let i = 0; i < editor.document.lineCount; i++) {
    const line = editor.document.lineAt(i);
    const lineText = line.text;
    const lineOffset = editor.document.offsetAt(line.range.start); // Offset of the start of the line

    const commentIndex = lineText.indexOf('#');

    let instructionPart = lineText;
    let commentPart = '';
    let commentPartOffset = 0; // Offset of the comment part relative to the start of the line

    if (commentIndex !== -1) {
      instructionPart = lineText.substring(0, commentIndex);
      commentPart = lineText.substring(commentIndex);
      commentPartOffset = commentIndex;
    }

    // 1. Find registers in backticks within the comment part
    if (commentPart) {
      let match: RegExpExecArray | null;
      // eslint-disable-next-line no-cond-assign
      while ((match = backtickRegisterRegex.exec(commentPart)) !== null) {
        const registerNumStr = match[1];
        if (!registerNumStr)
          continue;
        const registerNum = Number.parseInt(registerNumStr, 10);

        // Calculate the actual start position within the full document text
        // match.index is relative to commentPart start
        const numStartIndexInDocument = lineOffset + commentPartOffset + match.index + 1; // +1 to skip the opening backtick
        const numLength = registerNumStr.length;

        addRegisterDecoration(editor, numStartIndexInDocument, numLength, registerNum, collections);
      }
    }

    // 2. Find standalone registers within the instruction part
    let match: RegExpExecArray | null;
    // eslint-disable-next-line no-cond-assign
    while ((match = instructionRegisterRegex.exec(instructionPart)) !== null) {
      const registerNumStr = match[1];
      if (!registerNumStr)
        continue;
      const registerNum = Number.parseInt(registerNumStr, 10);

      // Calculate the actual start position within the full document text
      // match.index is relative to instructionPart start (which is line start)
      const numStartIndexInDocument = lineOffset + match.index;
      const numLength = registerNumStr.length;

      // Check if this is not part of an operator (already handled by collectAssociatedRegisters)
      // Use the original full text for this check
      const prevChar = numStartIndexInDocument > 0 ? text.charAt(numStartIndexInDocument - 1) : '';
      if (prevChar !== '*' && prevChar !== '=') {
        addRegisterDecoration(editor, numStartIndexInDocument, numLength, registerNum, collections);
      }
    }
  }
}

/**
 * Apply all collected decorations to the editor
 */
function applyDecorations(editor: vscode.TextEditor, decorations: DecorationCollections) {
  // Keep track of register numbers that have decorations in this update
  const appliedRegisters = new Set<number>();

  // Apply register decorations - create new decorations as needed
  for (const [registerNum, registerDecorationOptions] of decorations.registers.entries()) {
    const decoration = getRegisterDecoration(registerNum);
    editor.setDecorations(decoration, registerDecorationOptions);
    appliedRegisters.add(registerNum); // Mark this register as applied
  }

  // Clear decorations for registers that were previously decorated but are not in the current text
  for (const [registerNum, decoration] of registerDecorations.entries()) {
    if (!appliedRegisters.has(registerNum)) {
      // This register had decorations before, but not anymore. Clear them.
      editor.setDecorations(decoration, []);
    }
  }
}

/**
 * Clear all decorations from the editor
 */
function clearDecorations(editor: vscode.TextEditor) {
  // Clear all register decorations
  for (const decoration of registerDecorations.values()) {
    editor.setDecorations(decoration, []);
  }
  // No need to clear non-register decorations as they are not used
}

/**
 * Dispose all decorations
 */
export function disposeDecorations() {
  // Dispose all register decorations
  for (const decoration of registerDecorations.values()) {
    decoration.dispose();
  }
  registerDecorations.clear();
  // No need to dispose non-register decorations
}

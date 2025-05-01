import * as vscode from 'vscode';

// Define decoration types
let operatorDecoration: vscode.TextEditorDecorationType;
let indirectOperatorDecoration: vscode.TextEditorDecorationType;
let immediateOperatorDecoration: vscode.TextEditorDecorationType;
let bracketOpenDecoration: vscode.TextEditorDecorationType;
let bracketCloseDecoration: vscode.TextEditorDecorationType;

// Registry decoration types (dynamically created for registers)
const registerDecorations = new Map<number, vscode.TextEditorDecorationType>();

// Base pastel colors for non-register decorations
const pastelColors = {
  gray: { bg: 'rgba(220, 220, 220, 0.3)', border: 'rgba(180, 180, 180, 0.5)' },
  blue: { bg: 'rgba(173, 216, 230, 0.3)', border: 'rgba(135, 206, 235, 0.5)' },
  green: { bg: 'rgba(152, 251, 152, 0.3)', border: 'rgba(144, 238, 144, 0.5)' },
  gold: { bg: 'rgba(250, 218, 94, 0.3)', border: 'rgba(238, 232, 170, 0.5)' },
  pink: { bg: 'rgba(255, 182, 193, 0.3)', border: 'rgba(255, 105, 180, 0.5)' },
  purple: { bg: 'rgba(221, 160, 221, 0.3)', border: 'rgba(186, 85, 211, 0.5)' },
};

/**
 * Generates a pastel color for a register based on its number
 * Uses the golden ratio to distribute colors evenly around the color wheel
 */
function generateRegisterColor(registerNum: number): { bg: string; border: string; text: string } {
  // Use the golden ratio to create aesthetic color distribution
  const goldenRatioConjugate = 0.618033988749895;
  const hue = (registerNum * goldenRatioConjugate) % 1;

  // Convert to HSL color - pastel colors have high lightness and medium saturation
  const h = Math.floor(hue * 360);
  const s = 70; // Medium saturation for pastels
  const l = 85; // High lightness for pastels

  return {
    bg: `hsla(${h}, ${s}%, ${l}%, 0.3)`,
    border: `hsla(${h}, ${s}%, ${l - 10}%, 0.5)`,
    text: `hsla(${h}, ${s}%, ${l - 30}%, 1)`, // Darker text for contrast
  };
}

/**
 * Creates a text editor decoration type for a specific register
 */
function createRegisterDecoration(registerNum: number): vscode.TextEditorDecorationType {
  const colors = generateRegisterColor(registerNum);

  return vscode.window.createTextEditorDecorationType({
    backgroundColor: colors.bg,
    border: `1px solid ${colors.border}`,
    fontWeight: 'bold',
    rangeBehavior: vscode.DecorationRangeBehavior.ClosedClosed,
    before: {
      contentText: `R${registerNum}`,
      margin: '0 -0.1em 0 0',
      color: colors.text,
      backgroundColor: `${colors.bg.replace('0.3', '0.5')}`,
      border: `1px solid ${colors.border}`,
    },
    textDecoration: 'none; opacity: 0;', // Hide the original number
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
  // Create decoration types with proper styling
  operatorDecoration = vscode.window.createTextEditorDecorationType({
    backgroundColor: pastelColors.gray.bg,
    border: `1px solid ${pastelColors.gray.border}`,
    borderRadius: '3px',
    fontWeight: 'bold',
    rangeBehavior: vscode.DecorationRangeBehavior.ClosedClosed,
  });

  indirectOperatorDecoration = vscode.window.createTextEditorDecorationType({
    backgroundColor: pastelColors.green.bg,
    border: `1px solid ${pastelColors.green.border}`,
    borderRadius: '3px',
    fontWeight: 'bold',
    rangeBehavior: vscode.DecorationRangeBehavior.ClosedClosed,
  });

  immediateOperatorDecoration = vscode.window.createTextEditorDecorationType({
    backgroundColor: pastelColors.blue.bg,
    border: `1px solid ${pastelColors.blue.border}`,
    borderRadius: '3px',
    fontWeight: 'bold',
    rangeBehavior: vscode.DecorationRangeBehavior.ClosedClosed,
  });

  bracketOpenDecoration = vscode.window.createTextEditorDecorationType({
    backgroundColor: pastelColors.gold.bg,
    border: `1px solid ${pastelColors.gold.border}`,
    borderRadius: '3px 0 0 3px',
    fontWeight: 'bold',
    rangeBehavior: vscode.DecorationRangeBehavior.ClosedClosed,
  });

  bracketCloseDecoration = vscode.window.createTextEditorDecorationType({
    backgroundColor: pastelColors.gold.bg,
    border: `1px solid ${pastelColors.gold.border}`,
    borderRadius: '0 3px 3px 0',
    fontWeight: 'bold',
    rangeBehavior: vscode.DecorationRangeBehavior.ClosedClosed,
  });

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
    `RAM operator decorations ${!currentValue ? 'enabled' : 'disabled'}`,
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
 * Types to store decoration data
 */
interface DecorationCollections {
  operators: vscode.DecorationOptions[];
  indirectOperators: vscode.DecorationOptions[];
  immediateOperators: vscode.DecorationOptions[];
  bracketOpens: vscode.DecorationOptions[];
  bracketCloses: vscode.DecorationOptions[];
  registers: Map<number, vscode.DecorationOptions[]>;
}

/**
 * Collect all decorations from the text
 */
function collectDecorations(editor: vscode.TextEditor, text: string): DecorationCollections {
  const collections: DecorationCollections = {
    operators: [],
    indirectOperators: [],
    immediateOperators: [],
    bracketOpens: [],
    bracketCloses: [],
    registers: new Map(),
  };

  // Find all operators
  collectOperators(editor, text, collections);

  // Find standalone register numbers (not part of operators)
  collectStandaloneRegisters(editor, text, collections);

  return collections;
}

/**
 * Collect operators from the text
 */
function collectOperators(
  editor: vscode.TextEditor,
  text: string,
  collections: DecorationCollections,
) {
  const operatorRegex = /[*=[\]]/g;
  let match: RegExpExecArray | null;

  // eslint-disable-next-line no-cond-assign
  while ((match = operatorRegex.exec(text)) !== null) {
    const startPos = editor.document.positionAt(match.index);
    const endPos = editor.document.positionAt(match.index + match[0].length);
    const range = new vscode.Range(startPos, endPos);

    // Categorize by operator type
    switch (match[0]) {
      case '*':
        processStarOperator(editor, text, match, range, collections);
        break;
      case '=':
        processEqualsOperator(editor, text, match, range, collections);
        break;
      case '[':
        collections.bracketOpens.push({
          range,
          hoverMessage: new vscode.MarkdownString('**Opening bracket**'),
        });
        break;
      case ']':
        collections.bracketCloses.push({
          range,
          hoverMessage: new vscode.MarkdownString('**Closing bracket**'),
        });
        break;
    }
  }
}

/**
 * Process a star operator (*) - either multiplication or indirect addressing
 */
function processStarOperator(
  editor: vscode.TextEditor,
  text: string,
  match: RegExpExecArray,
  range: vscode.Range,
  collections: DecorationCollections,
) {
  const afterMatchIndex = match.index + 1;

  if (afterMatchIndex < text.length) {
    const afterMatch = text.substring(afterMatchIndex, afterMatchIndex + 1);

    if (/\d/.test(afterMatch)) {
      const registerNum = Number.parseInt(afterMatch, 10);

      // Create decoration for the operator
      collections.indirectOperators.push({
        range,
        hoverMessage: new vscode.MarkdownString(`**Indirect addressing mode** (Register ${registerNum})`),
      });

      // Create decoration for the register number
      addRegisterDecoration(editor, afterMatchIndex, registerNum, collections);
    }
    else {
      collections.operators.push({
        range,
        hoverMessage: new vscode.MarkdownString('**Multiplication operator**'),
      });
    }
  }
  else {
    collections.operators.push({
      range,
      hoverMessage: new vscode.MarkdownString('**Multiplication operator**'),
    });
  }
}

/**
 * Process an equals operator (=) - either assignment or immediate addressing
 */
function processEqualsOperator(
  editor: vscode.TextEditor,
  text: string,
  match: RegExpExecArray,
  range: vscode.Range,
  collections: DecorationCollections,
) {
  const afterMatchIndex = match.index + 1;

  if (afterMatchIndex < text.length) {
    const afterMatch = text.substring(afterMatchIndex, afterMatchIndex + 1);

    if (/\d/.test(afterMatch)) {
      const registerNum = Number.parseInt(afterMatch, 10);

      // Create decoration for the operator
      collections.immediateOperators.push({
        range,
        hoverMessage: new vscode.MarkdownString(`**Immediate addressing mode** (Register ${registerNum})`),
      });

      // Create decoration for the register number
      addRegisterDecoration(editor, afterMatchIndex, registerNum, collections);
    }
    else {
      collections.operators.push({
        range,
        hoverMessage: new vscode.MarkdownString('**Assignment operator**'),
      });
    }
  }
  else {
    collections.operators.push({
      range,
      hoverMessage: new vscode.MarkdownString('**Assignment operator**'),
    });
  }
}

/**
 * Add a register decoration to the collections
 */
function addRegisterDecoration(
  editor: vscode.TextEditor,
  index: number,
  registerNum: number,
  collections: DecorationCollections,
) {
  const registerStartPos = editor.document.positionAt(index);
  const registerEndPos = editor.document.positionAt(index + 1);
  const registerRange = new vscode.Range(registerStartPos, registerEndPos);

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
 */
function collectStandaloneRegisters(
  editor: vscode.TextEditor,
  text: string,
  collections: DecorationCollections,
) {
  const registerRegex = /\b\d\b/g;
  let match: RegExpExecArray | null;

  // eslint-disable-next-line no-cond-assign
  while ((match = registerRegex.exec(text)) !== null) {
    const registerNum = Number.parseInt(match[0], 10);
    const startPos = editor.document.positionAt(match.index);
    const endPos = editor.document.positionAt(match.index + match[0].length);
    const range = new vscode.Range(startPos, endPos);

    // Check if this is not part of an operator (already handled)
    const prevChar = match.index > 0 ? text.charAt(match.index - 1) : '';
    if (prevChar !== '*' && prevChar !== '=') {
      const decoration: vscode.DecorationOptions = {
        range,
        hoverMessage: new vscode.MarkdownString(`**Register ${registerNum}**`),
      };

      if (!collections.registers.has(registerNum)) {
        collections.registers.set(registerNum, []);
      }

      collections.registers.get(registerNum)!.push(decoration);
    }
  }
}

/**
 * Apply all collected decorations to the editor
 */
function applyDecorations(editor: vscode.TextEditor, decorations: DecorationCollections) {
  // Apply basic decorations
  editor.setDecorations(operatorDecoration, decorations.operators);
  editor.setDecorations(indirectOperatorDecoration, decorations.indirectOperators);
  editor.setDecorations(immediateOperatorDecoration, decorations.immediateOperators);
  editor.setDecorations(bracketOpenDecoration, decorations.bracketOpens);
  editor.setDecorations(bracketCloseDecoration, decorations.bracketCloses);

  // Apply register decorations - create new decorations as needed
  for (const [registerNum, registerDecorationOptions] of decorations.registers.entries()) {
    const decoration = getRegisterDecoration(registerNum);
    editor.setDecorations(decoration, registerDecorationOptions);
  }
}

/**
 * Clear all decorations from the editor
 */
function clearDecorations(editor: vscode.TextEditor) {
  // Clear basic decorations
  editor.setDecorations(operatorDecoration, []);
  editor.setDecorations(indirectOperatorDecoration, []);
  editor.setDecorations(immediateOperatorDecoration, []);
  editor.setDecorations(bracketOpenDecoration, []);
  editor.setDecorations(bracketCloseDecoration, []);

  // Clear all register decorations
  for (const decoration of registerDecorations.values()) {
    editor.setDecorations(decoration, []);
  }
}

/**
 * Dispose all decorations
 */
export function disposeDecorations() {
  // Dispose basic decorations
  operatorDecoration.dispose();
  indirectOperatorDecoration.dispose();
  immediateOperatorDecoration.dispose();
  bracketOpenDecoration.dispose();
  bracketCloseDecoration.dispose();

  // Dispose all register decorations
  for (const decoration of registerDecorations.values()) {
    decoration.dispose();
  }
  registerDecorations.clear();
}

import type * as Monaco from 'monaco-editor';
import { RAMTokenProvider } from '@ram/grammar/monaco';
import { textmateThemeToMonacoTheme } from '@shikijs/monaco';

import catppuccinMocha from '@shikijs/themes/catppuccin-mocha';

import { normalizeTheme } from '~/lib/theme';

export const RAM_LANG_CONFIG: Monaco.languages.ILanguageExtensionPoint = {
  id: 'ram',
  extensions: ['.ram'],
  aliases: ['ram', 'RAM'],
  mimetypes: ['text/x-ram'],
};

export const EDITOR_OPTIONS: Monaco.editor.IStandaloneEditorConstructionOptions = {
  fontSize: 16,
  fontWeight: 'bold',
  fontFamily: 'Geist Mono',
  fontLigatures: true,
  cursorBlinking: 'expand',
  cursorStyle: 'line',
  cursorSmoothCaretAnimation: 'on',
  padding: { top: 16, bottom: 16 },
  glyphMargin: true,
  lineNumbers: 'on',
  lineDecorationsWidth: 0,
  lineNumbersMinChars: 3,
};

export function configureMonaco(monaco: typeof Monaco) {
  registerLanguage(monaco);
  configureTheme(monaco);
}

export function registerLanguage(monaco: typeof Monaco) {
  monaco.languages.register(RAM_LANG_CONFIG);
  monaco.languages.setTokensProvider('ram', new RAMTokenProvider());
}

export function configureTheme(monaco: typeof Monaco) {
  const normalizedTheme = normalizeTheme(catppuccinMocha);
  const theme = textmateThemeToMonacoTheme(normalizedTheme);

  monaco.editor.defineTheme('catppuccin-mocha', theme);
}

import type * as Monaco from 'monaco-editor';
import catppuccinMocha from '@shikijs/themes/catppuccin-mocha';
import { textmateThemeToMonacoTheme } from '@shikijs/monaco';
import { normalizeTheme } from '~/lib/theme';
import { RAMTokenProvider } from '@ram/grammar/monaco';

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
  monaco.languages.register({ id: 'ram' });
  monaco.languages.setTokensProvider('ram', new RAMTokenProvider());

  const normalizedTheme = normalizeTheme(catppuccinMocha);
  const theme = textmateThemeToMonacoTheme(normalizedTheme);

  monaco.editor.defineTheme('catppuccin-mocha', theme);
  monaco.editor.setTheme('catppuccin-mocha');
}

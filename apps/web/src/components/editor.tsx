'use client';

import './editor-styles.css';

import type * as Monaco from 'monaco-editor';
import Editor, { useMonaco } from '@monaco-editor/react';
import { useCallback, useState } from 'react';

import { configureMonaco, EDITOR_OPTIONS } from '~/lib/config';
import { EXAMPLE_FILE } from '~/lib/consts';
import { useBreakpoints, useEditorDecorations } from '~/lib/hooks';

export function LiveEditor() {
  const monaco = useMonaco();
  const [editor, setEditor] = useState<Monaco.editor.IStandaloneCodeEditor | null>(null);
  const {
    breakpoints,
    hoverLine,
    toggleBreakpoint,
    handleMouseMove,
    handleMouseLeave,
  } = useBreakpoints();

  useEditorDecorations(editor, monaco, breakpoints, hoverLine);

  const handleBeforeMount = useCallback((monaco: typeof Monaco) => {
    configureMonaco(monaco);
  }, []);

  const handleEditorDidMount = useCallback(
    (editor: Monaco.editor.IStandaloneCodeEditor, monaco: typeof Monaco) => {
      setEditor(editor);

      editor.onMouseDown((e) => {
        if (e.target.type === monaco.editor.MouseTargetType.GUTTER_GLYPH_MARGIN) {
          e.event.preventDefault();
          if (e.target.position?.lineNumber)
            toggleBreakpoint(e.target.position.lineNumber);
        }
      });

      editor.onMouseMove((e) => {
        if (
          e.target.type === monaco.editor.MouseTargetType.GUTTER_GLYPH_MARGIN
          || e.target.type === monaco.editor.MouseTargetType.GUTTER_LINE_NUMBERS
        ) {
          handleMouseMove(e.target.position);
        }
        else {
          handleMouseLeave();
        }
      });

      editor.onMouseLeave(handleMouseLeave);
    },
    [toggleBreakpoint, handleMouseMove, handleMouseLeave],
  );

  if (!monaco)
    return null;

  return (
    <>
      <Editor
        height="100vh"
        width="100vw"
        theme="catppuccin-mocha"
        defaultLanguage="ram"
        defaultValue={EXAMPLE_FILE}
        beforeMount={handleBeforeMount}
        onMount={handleEditorDidMount}
        options={EDITOR_OPTIONS}
      />
    </>
  );
}

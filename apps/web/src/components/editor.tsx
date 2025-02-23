'use client';

import type * as Monaco from 'monaco-editor';
import Editor, { useMonaco } from '@monaco-editor/react';
import { useCallback, useState } from 'react';

import { EXAMPLE_FILE } from '~/lib/consts';
import { useBreakpoints, useEditorDecorations } from '~/lib/hooks';
import { EDITOR_OPTIONS, configureMonaco } from '~/lib/config';
import { EditorStyles } from './editor-styles';

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

  const handleEditorDidMount = useCallback(
    (editor: Monaco.editor.IStandaloneCodeEditor, monaco: typeof Monaco) => {
      setEditor(editor);
      configureMonaco(monaco);

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
      <EditorStyles />
      <Editor
        height="100vh"
        width="100vw"
        theme="vs-dark"
        defaultLanguage="ram"
        defaultValue={EXAMPLE_FILE}
        onMount={handleEditorDidMount}
        options={EDITOR_OPTIONS}
      />
    </>
  );
}

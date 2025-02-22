'use client';

import type * as Monaco from 'monaco-editor';
import Editor, { useMonaco } from '@monaco-editor/react';
import { useCallback, useEffect, useRef, useState, useMemo } from 'react';

import { RAMTokenProvider } from '@ram/grammar/monaco'

import { EXAMPLE_FILE } from '~/lib/consts';

export function LiveEditor() {
  const monaco = useMonaco();
  const [editor, setEditor]
    = useState<Monaco.editor.IStandaloneCodeEditor | null>(null);
  const [breakpoints, setBreakpoints] = useState<number[]>([]);
  const [hoverLine, setHoverLine] = useState<number | null>(null);
  const decorationsCollection = useRef<Monaco.editor.IEditorDecorationsCollection | null>(null);

  const handleEditorDidMount = useCallback(
    (editor: Monaco.editor.IStandaloneCodeEditor, monaco: typeof Monaco) => {
      setEditor(editor);
      decorationsCollection.current = editor.createDecorationsCollection();

      // Add click handler for breakpoints
      editor.onMouseDown((e) => {
        if (e.target.type === monaco.editor.MouseTargetType.GUTTER_GLYPH_MARGIN) {
          e.event.preventDefault();

          const lineNumber = e.target.position?.lineNumber;
          if (!lineNumber)
            return;

          setBreakpoints((prev) => {
            const idx = prev.indexOf(lineNumber);
            if (idx === -1) {
              return [...prev, lineNumber];
            }
            return prev.filter(bp => bp !== lineNumber);
          });
        }
      });

      editor.onMouseMove((e) => {
        if (
          e.target.type === monaco.editor.MouseTargetType.GUTTER_GLYPH_MARGIN
          || e.target.type === monaco.editor.MouseTargetType.GUTTER_LINE_NUMBERS
        ) {
          setHoverLine(e.target.position?.lineNumber || null);
        }
        else {
          setHoverLine(null);
        }
      });

      editor.onMouseLeave(() => {
        setHoverLine(null);
      });

      monaco.languages.register({ id: 'ram' })
      monaco.languages.setTokensProvider('ram', new RAMTokenProvider())
      monaco.editor.defineTheme('ram-theme', {
        base: 'vs-dark',
        inherit: true,
        colors: {},
        rules: [
          { token: 'comment.ram', foreground: '#585b70' }, // Catppuccin Surface2
          { token: 'number.ram', foreground: '#f9e2af' },  // Catppuccin Yellow
          { token: 'identifier.ram', foreground: '#89b4fa' }, // Catppuccin Blue
          { token: 'lbracket.ram', foreground: '#cdd6f4' }, // Catppuccin Text
          { token: 'rbracket.ram', foreground: '#cdd6f4' }, // Catppuccin Text
          { token: 'error.ram', foreground: '#f38ba8' }, // Catppuccin Red
        ],
      });
      monaco.editor.setTheme('ram-theme');
    },
    [],
  );

  // Update decorations whenever breakpoints change
  useEffect(() => {
    if (!editor || !monaco || !decorationsCollection.current)
      return;

    const model = editor.getModel();
    if (!model)
      return;

    const newDecorations: Monaco.editor.IModelDeltaDecoration[] = [];

    breakpoints.forEach((lineNumber) => {
      newDecorations.push({
        range: new monaco.Range(lineNumber, 1, lineNumber, 1),
        options: {
          isWholeLine: true,
          glyphMarginClassName: 'breakpoint-glyph',
          glyphMargin: {
            position: monaco.editor.GlyphMarginLane.Left,
          },
        },
      });
    });

    if (hoverLine !== null && !breakpoints.includes(hoverLine)) {
      newDecorations.push({
        range: new monaco.Range(hoverLine, 1, hoverLine, 1),
        options: {
          isWholeLine: true,
          glyphMarginClassName: 'breakpoint-glyph-hover',
          glyphMargin: {
            position: monaco.editor.GlyphMarginLane.Left,
          },
        },
      });
    }

    decorationsCollection.current.set(newDecorations);
  }, [editor, breakpoints, monaco, hoverLine]);

  // Dispose of decorations when the component unmounts or the model changes
  useEffect(() => {
    return () => {
      if (decorationsCollection.current) {
        decorationsCollection.current.clear();
      }
    };
  }, []);

  if (!monaco)
    return null;

  return (
    <>
      <style jsx global>
        {`
        .monaco-editor .margin-view-overlays .cgmr {
          cursor: pointer;
        }
        .breakpoint-glyph::before {
          content: '';
          display: block;
          position: absolute;
          left: 50%; /* Center the dot horizontally */
          top: 50%;
          transform: translate(-50%, -50%); /* Center precisely */
          background-color: #ff0000;
          border-radius: 50%;
          width: 10px;
          height: 10px;
        }
        .breakpoint-glyph-hover::before {
          content: '';
          display: block;
          position: absolute;
          left: 50%; /* Center the dot horizontally */
          top: 50%;
          transform: translate(-50%, -50%); /* Center precisely */
          background-color: rgba(255, 0, 0, 0.4);
          border-radius: 50%;
          width: 10px;
          height: 10px;
        }
      `}
      </style>
      <Editor
        height="100vh"
        width="100vw"
        theme="vs-dark"
        defaultLanguage="ram"
        defaultValue={EXAMPLE_FILE}
        onMount={handleEditorDidMount}
        options={{
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
        }}
      />
    </>
  );
}

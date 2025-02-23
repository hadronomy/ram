import type * as Monaco from 'monaco-editor';
import { useEffect, useRef, useState } from 'react';

export function useBreakpoints() {
  const [breakpoints, setBreakpoints] = useState<number[]>([]);
  const [hoverLine, setHoverLine] = useState<number | null>(null);

  const toggleBreakpoint = (lineNumber: number) => {
    setBreakpoints((prev) => {
      const idx = prev.indexOf(lineNumber);
      if (idx === -1) {
        return [...prev, lineNumber];
      }
      return prev.filter(bp => bp !== lineNumber);
    });
  };

  const handleMouseMove = (position: Monaco.Position | undefined) => {
    setHoverLine(position?.lineNumber || null);
  };

  const handleMouseLeave = () => {
    setHoverLine(null);
  };

  return {
    breakpoints,
    hoverLine,
    toggleBreakpoint,
    handleMouseMove,
    handleMouseLeave,
  };
}

export function useEditorDecorations(
  editor: Monaco.editor.IStandaloneCodeEditor | null,
  monaco: typeof Monaco | null,
  breakpoints: number[],
  hoverLine: number | null,
) {
  const decorationsCollection = useRef<Monaco.editor.IEditorDecorationsCollection | null>(null);

  useEffect(() => {
    if (!editor || !monaco)
      return;

    decorationsCollection.current = editor.createDecorationsCollection();

    return () => {
      if (decorationsCollection.current) {
        decorationsCollection.current.clear();
      }
    };
  }, [editor, monaco]);

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

  return decorationsCollection;
}

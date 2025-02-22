'use client';

import type * as Monaco from 'monaco-editor';
import Editor, { useMonaco } from '@monaco-editor/react';
import { useCallback, useEffect, useRef, useState } from 'react';

const EXAMPLE = `# Bubble Sort - Initial setup
        read 0      # Read size into R0
        store 10    # Store size in R10 (permanent)
        load =0
        store 2     # Initialize array index to 0

# Read array elements
read_loop: load 10
        sub 2
        jzero end_read
        read 0      # Read next element
        store 3[2]  # Store in array
        load 2
        add =1
        store 2     # Increment index
        jump read_loop

# Initialize sorting
end_read: load 10
        sub =1
        store 1     # n-1 in R1 (outer loop counter)

outer:  load 1      # Check if outer loop done
        jzero end_outer
        load =0
        store 2     # j = 0 (inner loop counter)

inner:  load 1
        sub 2       # Check if inner loop done
        jzero next_outer
        load 3[2]   # Load current element
        store 4     # Store in R4
        load 2
        add =1
        store 5     # Index for next element
        load 3[5]   # Load next element
        sub 4       # Compare next - current
        jgtz next_inner  # If next > current, no swap needed

# Swap elements
        load 3[5]   # Load next element
        store 6     # Store temporarily
        load 4      # Load current element
        store 3[5]  # Store current in next position
        load 6      # Load saved next element
        store 3[2]  # Store in current position

next_inner:
        load 2
        add =1
        store 2     # j++
        jump inner

next_outer:
        load 1
        sub =1
        store 1     # Decrement outer loop counter
        jump outer

# Print sorted array
end_outer: load =0
        store 2     # Reset counter
print:  load 10
        sub 2
        jzero terminate
        load 3[2]
        write 0
        load 2
        add =1
        store 2
        jump print

terminate: halt
`;

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
        defaultLanguage="txt"
        defaultValue={EXAMPLE}
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

import { useEffect, useRef } from 'preact/hooks';
import type { Signal, ReadonlySignal } from '@preact/signals';
import Prism from 'prismjs';
import 'prismjs/components/prism-rust';
import 'prismjs/components/prism-typescript';
import 'prismjs/components/prism-json';
import 'prismjs/components/prism-yaml';
import 'prismjs/components/prism-toml';

export interface CodeViewerProps {
  /** Signal for current filename (or null if nothing selected) */
  file: Signal<string | null> | ReadonlySignal<string | null>;
  /** Signal for file content */
  content: Signal<string> | ReadonlySignal<string>;
  /** Signal for line number to highlight (optional) */
  highlightLine?: Signal<number | null> | ReadonlySignal<number | null>;
  /** Placeholder message when no file selected */
  placeholderMessage?: string;
  /** Placeholder icon */
  placeholderIcon?: string;
}

function getLanguage(filename: string | null): string {
  if (!filename) return 'plaintext';
  if (filename.endsWith('.rs')) return 'rust';
  if (filename.endsWith('.ts') || filename.endsWith('.tsx')) return 'typescript';
  if (filename.endsWith('.js') || filename.endsWith('.jsx')) return 'javascript';
  if (filename.endsWith('.json')) return 'json';
  if (filename.endsWith('.toml')) return 'toml';
  if (filename.endsWith('.yaml') || filename.endsWith('.yml')) return 'yaml';
  if (filename.endsWith('.md')) return 'markdown';
  return 'plaintext';
}

export function CodeViewer({ 
  file, 
  content, 
  highlightLine,
  placeholderMessage = 'Click a source file to view code',
  placeholderIcon = '📄',
}: CodeViewerProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const highlightLineRef = useRef<HTMLDivElement>(null);
  
  const filename = file.value;
  const fileContent = content.value;
  const lineToHighlight = highlightLine?.value ?? null;
  const language = getLanguage(filename);
  
  useEffect(() => {
    // Scroll to highlighted line
    if (highlightLineRef.current) {
      highlightLineRef.current.scrollIntoView({ behavior: 'instant', block: 'center' });
    }
  }, [lineToHighlight, fileContent]);

  if (!filename) {
    return (
      <div class="code-viewer empty">
        <div class="placeholder-message">
          <span class="placeholder-icon">{placeholderIcon}</span>
          <p>{placeholderMessage}</p>
        </div>
      </div>
    );
  }

  const lines = fileContent.split('\n');
  
  // Highlight code with Prism
  let highlightedCode = fileContent;
  try {
    const grammar = Prism.languages[language];
    if (grammar) {
      highlightedCode = Prism.highlight(fileContent, grammar, language);
    }
  } catch {
    // Fall back to plain text
  }
  
  const highlightedLines = highlightedCode.split('\n');

  // Extract just the filename for display
  const displayName = filename.split('/').pop() || filename;

  return (
    <div class="code-viewer" ref={containerRef}>
      <div class="code-header">
        <span class="code-filename" title={filename}>{displayName}</span>
        <span class="code-language">{language}</span>
        <span class="code-lines">{lines.length} lines</span>
      </div>
      
      <div class="code-content">
        <pre class="code-pre">
          {highlightedLines.map((line, i) => {
            const lineNum = i + 1;
            const isHighlight = lineNum === lineToHighlight;
            return (
              <div 
                key={i}
                ref={isHighlight ? highlightLineRef : undefined}
                class={`code-line ${isHighlight ? 'highlight' : ''}`}
              >
                <span class="line-number">{lineNum}</span>
                <code dangerouslySetInnerHTML={{ __html: line || ' ' }} />
              </div>
            );
          })}
        </pre>
      </div>
    </div>
  );
}

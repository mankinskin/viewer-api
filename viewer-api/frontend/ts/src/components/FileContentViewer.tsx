/**
 * FileContentViewer — generic file content renderer.
 *
 * Selects the right renderer based on file extension:
 * - Custom renderers via `onRenderCustom` hook (highest priority)
 * - Built-in CodeViewer for source files (.rs, .ts, .js, .json, .toml, .yaml)
 * - CodeViewer as plaintext fallback for everything else
 *
 * Apps can override rendering for specific file types. For example:
 * - doc-viewer provides custom `.md` renderer (using marked/hljs)
 * - log-viewer provides custom `.log` renderer (using LogViewer)
 */
import { ComponentChildren } from 'preact';
import type { Signal, ReadonlySignal } from '@preact/signals';
import { CodeViewer } from './CodeViewer';

export interface FileContentViewerProps {
  /** Signal for current filename (or null if nothing selected) */
  file: Signal<string | null> | ReadonlySignal<string | null>;
  /** Signal for file content */
  content: Signal<string> | ReadonlySignal<string>;
  /** Signal for line number to highlight (optional) */
  highlightLine?: Signal<number | null> | ReadonlySignal<number | null>;
  /**
   * Custom render hook. Called with the filename and content.
   * Return a ComponentChildren to render, or `null` to fall back to built-in renderer.
   */
  onRenderCustom?: (filename: string, content: string) => ComponentChildren | null;
  /** Placeholder message when no file is selected */
  placeholderMessage?: string;
  /** Placeholder icon */
  placeholderIcon?: string;
}

export function FileContentViewer({
  file,
  content,
  highlightLine,
  onRenderCustom,
  placeholderMessage = 'Select a file to view',
  placeholderIcon = '📄',
}: FileContentViewerProps) {
  const filename = file.value;
  const fileContent = content.value;

  // Try custom renderer first
  if (onRenderCustom && filename && fileContent) {
    const custom = onRenderCustom(filename, fileContent);
    if (custom !== null) {
      return <>{custom}</>;
    }
  }

  // Fall back to CodeViewer (handles syntax highlighting for known languages, plaintext for others)
  return (
    <CodeViewer
      file={file}
      content={content}
      highlightLine={highlightLine}
      placeholderMessage={placeholderMessage}
      placeholderIcon={placeholderIcon}
    />
  );
}

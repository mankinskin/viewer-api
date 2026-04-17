import { JSX, ComponentChildren } from 'preact';

export interface LayoutProps {
  /** Header component/content */
  header: ComponentChildren;
  /** Sidebar component/content (optional) */
  sidebar?: ComponentChildren;
  /** Main content area */
  children: ComponentChildren;
  /** Optional class for the app wrapper */
  class?: string;
  /** Optional extra content between header and main (e.g., filter panels) */
  extraContent?: ComponentChildren;
}

/**
 * Common layout component for viewer tools.
 * 
 * Provides a consistent app structure:
 * ```
 * ┌─────────────────────────────────────┐
 * │              Header                 │
 * ├─────────────────────────────────────┤
 * │          Extra Content              │
 * ├───────────┬─────────────────────────┤
 * │           │                         │
 * │  Sidebar  │       Main Content      │
 * │           │                         │
 * └───────────┴─────────────────────────┘
 * ```
 */
export function Layout({ 
  header, 
  sidebar, 
  children, 
  class: className = '',
  extraContent 
}: LayoutProps): JSX.Element {
  return (
    <div class={`app ${className}`}>
      {header}
      {extraContent}
      <div class="main-layout">
        {sidebar}
        <main class="content">
          {children}
        </main>
      </div>
    </div>
  );
}

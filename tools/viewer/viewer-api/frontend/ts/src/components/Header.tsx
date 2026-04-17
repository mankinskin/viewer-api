import { JSX, ComponentChildren } from 'preact';

export interface HeaderProps {
  /** Title text displayed in the header */
  title: string;
  /** Optional icon element to display before the title */
  icon?: JSX.Element;
  /** Optional subtitle displayed after the title */
  subtitle?: string;
  /** Optional right-side content (search, buttons, etc.) */
  rightContent?: ComponentChildren;
  /** Optional middle content (between title and right) */
  middleContent?: ComponentChildren;
  /** Optional class name */
  class?: string;
  /** When provided, renders a hamburger menu button (shown on mobile via CSS) */
  onMenuToggle?: () => void;
}

/**
 * Common header component for viewer tools.
 * 
 * Provides a consistent header layout with:
 * - Left: (hamburger) + icon + title + subtitle
 * - Middle: optional content (search forms, etc.)
 * - Right: optional content (buttons, etc.)
 */
export function Header({ 
  title, 
  icon, 
  subtitle, 
  rightContent,
  middleContent,
  class: className = '',
  onMenuToggle,
}: HeaderProps): JSX.Element {
  return (
    <header class={`header ${className}`}>
      <div class="header-left">
        {onMenuToggle && (
          <button class="sidebar-hamburger" onClick={onMenuToggle} title="Toggle sidebar">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
              <line x1="3" y1="6" x2="21" y2="6" />
              <line x1="3" y1="12" x2="21" y2="12" />
              <line x1="3" y1="18" x2="21" y2="18" />
            </svg>
          </button>
        )}
        {icon && <span class="header-icon">{icon}</span>}
        <h1 class="header-title">{title}</h1>
        {subtitle && <span class="header-subtitle">{subtitle}</span>}
      </div>
      
      {middleContent && (
        <div class="header-middle">
          {middleContent}
        </div>
      )}
      
      {rightContent && (
        <div class="header-right">
          {rightContent}
        </div>
      )}
    </header>
  );
}

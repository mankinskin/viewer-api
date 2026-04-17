import { JSX } from 'preact';

export interface SpinnerProps {
  size?: 'sm' | 'md' | 'lg';
  class?: string;
}

export function Spinner({ size = 'md', class: className = '' }: SpinnerProps): JSX.Element {
  const sizeMap = {
    sm: 16,
    md: 24,
    lg: 32,
  };
  const s = sizeMap[size];

  return (
    <svg
      class={`spinner ${className}`}
      width={s}
      height={s}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      style={{ animation: 'spin 1s linear infinite' }}
    >
      <circle cx="12" cy="12" r="10" stroke-opacity="0.25" />
      <path d="M12 2a10 10 0 0 1 10 10" stroke-linecap="round" />
      <style>{`
        @keyframes spin {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }
      `}</style>
    </svg>
  );
}

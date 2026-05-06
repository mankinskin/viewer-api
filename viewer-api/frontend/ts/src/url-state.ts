/**
 * Generic URL hash-based state management for viewer tools.
 *
 * Provides pushState/popState routing with loop prevention so that:
 * - App state changes update the URL hash
 * - Browser back/forward navigates between states
 * - Page reload restores the last state from the URL
 * - URLs are shareable/linkable
 */

export interface UrlStateConfig<T> {
  /** Convert app state to a URL hash path (without the leading #). */
  stateToHash: (state: T) => string;
  /** Parse a URL hash path (without #) back to app state. Return null if invalid. */
  hashToState: (hash: string) => T | null;
  /** Called when the user navigates via back/forward or page load. Apply the state to the app. */
  onNavigate: (state: T) => void | Promise<void>;
  /** Return the current app state (used to avoid redundant navigations). */
  getCurrentState: () => T | null;
  /** Compare two states for equality. Defaults to JSON.stringify comparison. */
  statesEqual?: (a: T, b: T) => boolean;
}

export interface UrlStateManager<T> {
  /** Update the URL hash to reflect the given state. Call this when app state changes. */
  updateHash: (state: T) => void;
  /** Read the current state from the URL hash. Returns null if no/invalid hash. */
  getStateFromUrl: () => T | null;
  /** Start listening for hashchange/popstate events. Call once on app init. */
  initListener: () => void;
}

export function createUrlStateManager<T>(config: UrlStateConfig<T>): UrlStateManager<T> {
  let isNavigatingFromUrl = false;

  const statesEqual = config.statesEqual ?? ((a: T, b: T) =>
    JSON.stringify(a) === JSON.stringify(b)
  );

  function getStateFromUrl(): T | null {
    const hash = window.location.hash.slice(1); // remove #
    if (!hash) return null;
    return config.hashToState(hash);
  }

  function updateHash(state: T): void {
    if (isNavigatingFromUrl) return;

    const hashPath = config.stateToHash(state);
    const newUrl = `#${hashPath}`;
    if (window.location.hash !== newUrl) {
      window.history.pushState(null, '', newUrl);
    }
  }

  async function handleNavigation(): Promise<void> {
    const urlState = getStateFromUrl();
    if (!urlState) return;

    const current = config.getCurrentState();
    if (current && statesEqual(current, urlState)) return;

    isNavigatingFromUrl = true;
    try {
      await config.onNavigate(urlState);
    } finally {
      isNavigatingFromUrl = false;
    }
  }

  function initListener(): void {
    window.addEventListener('hashchange', handleNavigation);
    window.addEventListener('popstate', handleNavigation);
  }

  return { updateHash, getStateFromUrl, initListener };
}

// Session management utilities for viewer frontends
// Provides session ID generation, storage, and HTTP header injection

const SESSION_HEADER = 'x-session-id';
const SESSION_STORAGE_KEY = 'viewer_session_id';

/**
 * Get or create a session ID.
 * Uses sessionStorage to persist across page reloads within the same tab.
 */
export function getSessionId(): string {
  let sessionId = sessionStorage.getItem(SESSION_STORAGE_KEY);
  if (!sessionId) {
    sessionId = crypto.randomUUID();
    sessionStorage.setItem(SESSION_STORAGE_KEY, sessionId);
  }
  return sessionId;
}

/**
 * Clear the current session ID.
 * Useful for logout or session reset scenarios.
 */
export function clearSession(): void {
  sessionStorage.removeItem(SESSION_STORAGE_KEY);
}

/**
 * Add session header to fetch options.
 * Use this wrapper for all API calls that need session tracking.
 * 
 * @example
 * const response = await fetch('/api/data', withSession());
 * const response = await fetch('/api/data', withSession({ method: 'POST', body: '...' }));
 */
export function withSession(options: RequestInit = {}): RequestInit {
  return {
    ...options,
    headers: {
      ...options.headers,
      [SESSION_HEADER]: getSessionId(),
    },
  };
}

/**
 * Session configuration from the server
 */
export interface SessionConfig {
  session_id: string;
  verbose: boolean;
  data: Record<string, string>;
}

/**
 * Request type for updating session configuration
 */
export interface SessionConfigUpdate {
  verbose?: boolean;
  data?: Record<string, string>;
}

/**
 * Create a session API client for a given base URL
 */
export function createSessionApi(apiBase: string) {
  return {
    /**
     * Get current session configuration from the server
     */
    async getConfig(): Promise<SessionConfig> {
      const response = await fetch(`${apiBase}/session`, withSession());
      if (!response.ok) {
        throw new Error('Failed to get session config');
      }
      return response.json();
    },

    /**
     * Update session configuration on the server
     */
    async updateConfig(update: SessionConfigUpdate): Promise<SessionConfig> {
      const response = await fetch(`${apiBase}/session`, withSession({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(update),
      }));
      if (!response.ok) {
        throw new Error('Failed to update session config');
      }
      return response.json();
    },

    /**
     * Set a custom data value in the session
     */
    async setData(key: string, value: string): Promise<SessionConfig> {
      return this.updateConfig({ data: { [key]: value } });
    },

    /**
     * Get a custom data value from the session
     */
    async getData(key: string): Promise<string | undefined> {
      const config = await this.getConfig();
      return config.data[key];
    },
  };
}

// Export the header constant for use in custom implementations
export { SESSION_HEADER };

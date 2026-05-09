import type { ComponentChildren } from 'preact';
import { createContext } from 'preact';
import { useContext } from 'preact/hooks';

import type { ThemeSettingsStore } from '../../store/theme';

const ThemeSettingsContext = createContext<ThemeSettingsStore | null>(null);

export function useThemeSettingsStore(): ThemeSettingsStore {
  return useContext(ThemeSettingsContext)!;
}

export function ThemeSettingsProvider({
  store,
  children,
}: {
  store: ThemeSettingsStore;
  children: ComponentChildren;
}) {
  return (
    <ThemeSettingsContext.Provider value={store}>
      {children}
    </ThemeSettingsContext.Provider>
  );
}
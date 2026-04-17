/**
 * useNestingState — manages nesting view settings with localStorage persistence.
 *
 * Exposes the current NestingSettings and a setter. The computed parts of
 * NestingState (shells, duplicates) are derived by the nesting modules
 * each frame — they are not managed here.
 */
import { useState, useCallback } from 'preact/hooks';
import type { NestingSettings } from '../types';

const STORAGE_KEY = 'hg-nesting-settings';

const DEFAULT_SETTINGS: NestingSettings = {
    enabled: false,
    duplicateMode: false,
    parentDepth: 2,
    childDepth: 1,
};

function loadSettings(): NestingSettings {
    try {
        const raw = localStorage.getItem(STORAGE_KEY);
        if (raw) {
            const parsed = JSON.parse(raw);
            return {
                enabled: typeof parsed.enabled === 'boolean' ? parsed.enabled : DEFAULT_SETTINGS.enabled,
                duplicateMode: typeof parsed.duplicateMode === 'boolean' ? parsed.duplicateMode : DEFAULT_SETTINGS.duplicateMode,
                parentDepth: typeof parsed.parentDepth === 'number'
                    ? Math.max(1, Math.min(5, parsed.parentDepth))
                    : DEFAULT_SETTINGS.parentDepth,
                childDepth: typeof parsed.childDepth === 'number'
                    ? Math.max(1, Math.min(3, parsed.childDepth))
                    : DEFAULT_SETTINGS.childDepth,
            };
        }
    } catch { /* ignore corrupt storage */ }
    return { ...DEFAULT_SETTINGS };
}

function saveSettings(settings: NestingSettings): void {
    try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
    } catch { /* quota exceeded or private mode — ignore */ }
}

export interface NestingStateResult {
    nestingSettings: NestingSettings;
    setNestingSettings: (update: Partial<NestingSettings>) => void;
}

export function useNestingState(): NestingStateResult {
    const [settings, setSettings] = useState<NestingSettings>(loadSettings);

    const setNestingSettings = useCallback((update: Partial<NestingSettings>) => {
        setSettings(prev => {
            const next = { ...prev, ...update };
            // Clamp depth values
            next.parentDepth = Math.max(1, Math.min(5, next.parentDepth));
            next.childDepth = Math.max(1, Math.min(3, next.childDepth));
            saveSettings(next);
            return next;
        });
    }, []);

    return { nestingSettings: settings, setNestingSettings };
}

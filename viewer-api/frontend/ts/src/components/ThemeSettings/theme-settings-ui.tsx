import type { ComponentChildren } from 'preact';
import { useRef, useState } from 'preact/hooks';

import type { SavedTheme, ThemeColors } from '../../store/theme';
import { captureOverlayThumbnail } from '../WgpuOverlay/WgpuOverlay';
import { useThemeSettingsStore } from './theme-settings-context';

interface ColorRowProps {
  label: string;
  description?: string;
  colorKey: keyof ThemeColors;
}

export function ColorRow({ label, description, colorKey }: ColorRowProps) {
  const store = useThemeSettingsStore();
  const value = store.themeColors.value[colorKey];

  return (
    <div class="theme-color-row">
      <div class="theme-color-info">
        <span class="theme-color-label">{label}</span>
        {description && <span class="theme-color-desc">{description}</span>}
      </div>
      <div class="theme-color-controls">
        <input
          type="color"
          class="theme-color-picker"
          value={value}
          onInput={(event) =>
            store.updateColor(colorKey, (event.target as HTMLInputElement).value)
          }
        />
        <input
          type="text"
          class="theme-color-hex"
          value={value}
          maxLength={7}
          onInput={(event) => {
            const nextValue = (event.target as HTMLInputElement).value;
            if (/^#[0-9a-fA-F]{6}$/.test(nextValue)) {
              store.updateColor(colorKey, nextValue);
            }
          }}
        />
        <button
          class="theme-color-reset"
          title="Reset to default"
          onClick={() => store.updateColor(colorKey, store.defaultTheme[colorKey])}
        >
          ↺
        </button>
      </div>
    </div>
  );
}

interface SectionProps {
  title: string;
  icon: string;
  children: ComponentChildren;
  defaultOpen?: boolean;
  className?: string;
}

export function Section({
  title,
  icon,
  children,
  defaultOpen = false,
  className,
}: SectionProps) {
  const [open, setOpen] = useState(defaultOpen);

  return (
    <section class={`theme-section ${open ? 'open' : ''}${className ? ` ${className}` : ''}`}>
      <button class="theme-section-header" onClick={() => setOpen(!open)}>
        <span class="theme-section-icon">{icon}</span>
        <span class="theme-section-title">{title}</span>
        <span class="theme-section-chevron">{open ? '▾' : '▸'}</span>
      </button>
      {open && <div class="theme-section-body">{children}</div>}
    </section>
  );
}

export function SaveThemeButton() {
  const store = useThemeSettingsStore();
  const [showInput, setShowInput] = useState(false);
  const [saving, setSaving] = useState(false);
  const [name, setName] = useState('');
  const inputRef = useRef<HTMLInputElement>(null);

  async function handleSave() {
    const trimmed = name.trim();
    if (!trimmed || saving) return;

    setSaving(true);
    try {
      const thumbnail = await captureOverlayThumbnail();
      store.saveTheme(trimmed, thumbnail);
    } catch {
      store.saveTheme(trimmed);
    }
    setName('');
    setShowInput(false);
    setSaving(false);
  }

  if (!showInput) {
    return (
      <button
        class="btn btn-primary"
        onClick={() => {
          setShowInput(true);
          setTimeout(() => inputRef.current?.focus(), 0);
        }}
      >
        💾 Save Theme
      </button>
    );
  }

  return (
    <div class="save-theme-inline">
      <input
        ref={inputRef}
        type="text"
        class="save-theme-input"
        placeholder="Theme name…"
        value={name}
        maxLength={40}
        onInput={(event) => setName((event.target as HTMLInputElement).value)}
        onKeyDown={(event) => {
          if (event.key === 'Enter') void handleSave();
          if (event.key === 'Escape') {
            setShowInput(false);
            setName('');
          }
        }}
      />
      <button
        class="btn btn-primary"
        onClick={() => void handleSave()}
        disabled={!name.trim() || saving}
      >
        {saving ? '…' : 'Save'}
      </button>
      <button
        class="btn btn-secondary"
        onClick={() => {
          setShowInput(false);
          setName('');
        }}
      >
        ✕
      </button>
    </div>
  );
}

function SavedThemeCard({ theme }: { theme: SavedTheme }) {
  const store = useThemeSettingsStore();
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [confirmUpdate, setConfirmUpdate] = useState(false);
  const [updating, setUpdating] = useState(false);
  const [editing, setEditing] = useState(false);
  const [editName, setEditName] = useState(theme.name);

  async function handleUpdate() {
    setUpdating(true);
    try {
      const thumbnail = await captureOverlayThumbnail();
      store.updateSavedTheme(theme.id, thumbnail);
    } catch {
      store.updateSavedTheme(theme.id);
    }
    setUpdating(false);
    setConfirmUpdate(false);
  }

  function handleRename() {
    const trimmed = editName.trim();
    if (trimmed && trimmed !== theme.name) {
      store.renameSavedTheme(theme.id, trimmed);
    }
    setEditing(false);
  }

  const date = new Date(theme.createdAt);
  const dateStr = `${date.toLocaleDateString()} ${date.toLocaleTimeString([], {
    hour: '2-digit',
    minute: '2-digit',
  })}`;

  return (
    <div class="saved-theme-card">
      {theme.thumbnail ? (
        <img class="saved-theme-thumbnail" src={theme.thumbnail} alt={theme.name} />
      ) : (
        <div class="saved-theme-swatches">
          <span class="theme-preset-swatch" style={{ background: theme.colors.bgPrimary }} />
          <span class="theme-preset-swatch" style={{ background: theme.colors.accentOrange }} />
          <span class="theme-preset-swatch" style={{ background: theme.colors.accentBlue }} />
          <span class="theme-preset-swatch" style={{ background: theme.colors.levelError }} />
          <span class="theme-preset-swatch" style={{ background: theme.colors.cinderEmber }} />
          <span class="theme-preset-swatch" style={{ background: theme.colors.textPrimary }} />
        </div>
      )}
      <div class="saved-theme-info">
        {editing ? (
          <input
            type="text"
            class="save-theme-input saved-theme-rename"
            value={editName}
            maxLength={40}
            onInput={(event) =>
              setEditName((event.target as HTMLInputElement).value)
            }
            onKeyDown={(event) => {
              if (event.key === 'Enter') handleRename();
              if (event.key === 'Escape') {
                setEditing(false);
                setEditName(theme.name);
              }
            }}
            onBlur={handleRename}
            autoFocus
          />
        ) : (
          <strong
            class="saved-theme-name"
            onDblClick={() => setEditing(true)}
            title="Double-click to rename"
          >
            {theme.name}
          </strong>
        )}
        <span class="saved-theme-date">{dateStr}</span>
      </div>
      <div class="saved-theme-actions">
        <button
          class="btn btn-primary btn-sm"
          onClick={() => store.applySavedTheme(theme)}
          title="Apply this theme"
        >
          Apply
        </button>
        {confirmUpdate ? (
          <button
            class="btn btn-warn btn-sm"
            onClick={() => void handleUpdate()}
            disabled={updating}
          >
            {updating ? '…' : 'Confirm'}
          </button>
        ) : (
          <button
            class="btn btn-secondary btn-sm"
            onClick={() => setConfirmUpdate(true)}
            title="Overwrite with current colors"
          >
            ✏️
          </button>
        )}
        {confirmDelete ? (
          <button
            class="btn btn-danger btn-sm"
            onClick={() => {
              store.deleteTheme(theme.id);
              setConfirmDelete(false);
            }}
          >
            Confirm
          </button>
        ) : (
          <button
            class="btn btn-secondary btn-sm"
            onClick={() => setConfirmDelete(true)}
            title="Delete this theme"
          >
            🗑
          </button>
        )}
      </div>
    </div>
  );
}

export function SavedThemesPanel() {
  const store = useThemeSettingsStore();
  const themes = store.savedThemes.value;

  return (
    <div class="saved-themes-panel">
      <h3 class="saved-themes-title">Saved Themes</h3>
      <p class="saved-themes-subtitle">Your custom themes, stored in the browser.</p>
      {themes.length === 0 ? (
        <div class="saved-themes-empty">
          <span class="saved-themes-empty-icon">◇</span>
          <p>No saved themes yet.</p>
          <p class="saved-themes-empty-hint">
            Use the "💾 Save Theme" button to save your current color configuration.
          </p>
        </div>
      ) : (
        <div class="saved-themes-list">
          {themes.map((theme) => (
            <SavedThemeCard key={theme.id} theme={theme} />
          ))}
        </div>
      )}
    </div>
  );
}

export function ImportThemeButton() {
  const store = useThemeSettingsStore();
  const fileRef = useRef<HTMLInputElement>(null);
  const [error, setError] = useState<string | null>(null);

  async function handleFile(event: Event) {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;

    const nextError = await store.importTheme(file);
    setError(nextError ?? null);
    input.value = '';
  }

  return (
    <>
      <input
        ref={fileRef}
        type="file"
        accept=".json,application/json"
        style={{ display: 'none' }}
        onChange={(event) => void handleFile(event)}
      />
      <button
        class="btn btn-secondary"
        onClick={() => fileRef.current?.click()}
        title="Load a theme from a .json file"
      >
        📂 Import
      </button>
      {error && <span class="theme-import-error">{error}</span>}
    </>
  );
}
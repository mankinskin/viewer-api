/**
 * ThemeSettings — shared color/effect theme settings component.
 *
 * Parameterized via a `ThemeSettingsStore` prop so any viewer can use it with
 * its own signals and functions. The store is injected via Preact context so
 * sub-components don't need prop drilling.
 *
 * Sections:
 *   - Presets (one-click theme switching)
 *   - Backgrounds / Text / Borders / Accents
 *   - Log Level Colors + Badge Text
 *   - Span Badge Colors
 *   - GPU toggle
 *   - Particle Effects (Sparks, Embers, Beams, Glitter, Cinder)
 *   - Background Smoke
 *   - Glass Panels
 *   - CRT Effect
 *   - Saved Themes panel
 */
import { createContext } from 'preact';
import { useContext, useState, useRef } from 'preact/hooks';
import type { ThemeColors, ThemePreset, SavedTheme, ThemeSettingsStore } from '../../store/theme';
import { captureOverlayThumbnail, gpuOverlayEnabled } from '../WgpuOverlay/WgpuOverlay';
import './theme-settings.css';

// ── Context ──────────────────────────────────────────────────────────────────

const Ctx = createContext<ThemeSettingsStore | null>(null);
function useStore(): ThemeSettingsStore { return useContext(Ctx)!; }

// ── ColorRow ─────────────────────────────────────────────────────────────────

interface ColorRowProps {
  label: string;
  description?: string;
  colorKey: keyof ThemeColors;
}

function ColorRow({ label, description, colorKey }: ColorRowProps) {
  const store = useStore();
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
          onInput={(e) => store.updateColor(colorKey, (e.target as HTMLInputElement).value)}
        />
        <input
          type="text"
          class="theme-color-hex"
          value={value}
          maxLength={7}
          onInput={(e) => {
            const v = (e.target as HTMLInputElement).value;
            if (/^#[0-9a-fA-F]{6}$/.test(v)) store.updateColor(colorKey, v);
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

// ── Section ───────────────────────────────────────────────────────────────────

interface SectionProps {
  title: string;
  icon: string;
  children: preact.ComponentChildren;
  defaultOpen?: boolean;
  className?: string;
}

function Section({ title, icon, children, defaultOpen = false, className }: SectionProps) {
  const [open, setOpen] = useState(defaultOpen);
  return (
    <section class={`theme-section ${open ? 'open' : ''}${className ? ' ' + className : ''}`}>
      <button class="theme-section-header" onClick={() => setOpen(!open)}>
        <span class="theme-section-icon">{icon}</span>
        <span class="theme-section-title">{title}</span>
        <span class="theme-section-chevron">{open ? '▾' : '▸'}</span>
      </button>
      {open && <div class="theme-section-body">{children}</div>}
    </section>
  );
}

// ── SaveThemeButton ──────────────────────────────────────────────────────────

function SaveThemeButton() {
  const store = useStore();
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
      <button class="btn btn-primary" onClick={() => { setShowInput(true); setTimeout(() => inputRef.current?.focus(), 0); }}>
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
        onInput={(e) => setName((e.target as HTMLInputElement).value)}
        onKeyDown={(e) => {
          if (e.key === 'Enter') void handleSave();
          if (e.key === 'Escape') { setShowInput(false); setName(''); }
        }}
      />
      <button class="btn btn-primary" onClick={() => void handleSave()} disabled={!name.trim() || saving}>
        {saving ? '…' : 'Save'}
      </button>
      <button class="btn btn-secondary" onClick={() => { setShowInput(false); setName(''); }}>✕</button>
    </div>
  );
}

// ── SavedThemeCard ────────────────────────────────────────────────────────────

function SavedThemeCard({ theme }: { theme: SavedTheme }) {
  const store = useStore();
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
    if (trimmed && trimmed !== theme.name) store.renameSavedTheme(theme.id, trimmed);
    setEditing(false);
  }

  const date = new Date(theme.createdAt);
  const dateStr = `${date.toLocaleDateString()} ${date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}`;

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
            onInput={(e) => setEditName((e.target as HTMLInputElement).value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') handleRename();
              if (e.key === 'Escape') { setEditing(false); setEditName(theme.name); }
            }}
            onBlur={handleRename}
            // eslint-disable-next-line jsx-a11y/no-autofocus
            autoFocus
          />
        ) : (
          <strong class="saved-theme-name" onDblClick={() => setEditing(true)} title="Double-click to rename">
            {theme.name}
          </strong>
        )}
        <span class="saved-theme-date">{dateStr}</span>
      </div>
      <div class="saved-theme-actions">
        <button class="btn btn-primary btn-sm" onClick={() => store.applySavedTheme(theme)} title="Apply this theme">
          Apply
        </button>
        {confirmUpdate ? (
          <button class="btn btn-warn btn-sm" onClick={() => void handleUpdate()} disabled={updating}>
            {updating ? '…' : 'Confirm'}
          </button>
        ) : (
          <button class="btn btn-secondary btn-sm" onClick={() => setConfirmUpdate(true)} title="Overwrite with current colors">
            ✏️
          </button>
        )}
        {confirmDelete ? (
          <button class="btn btn-danger btn-sm" onClick={() => { store.deleteTheme(theme.id); setConfirmDelete(false); }}>
            Confirm
          </button>
        ) : (
          <button class="btn btn-secondary btn-sm" onClick={() => setConfirmDelete(true)} title="Delete this theme">
            🗑
          </button>
        )}
      </div>
    </div>
  );
}

// ── SavedThemesPanel ─────────────────────────────────────────────────────────

function SavedThemesPanel() {
  const store = useStore();
  const themes = store.savedThemes.value;
  return (
    <div class="saved-themes-panel">
      <h3 class="saved-themes-title">Saved Themes</h3>
      <p class="saved-themes-subtitle">Your custom themes, stored in the browser.</p>
      {themes.length === 0 ? (
        <div class="saved-themes-empty">
          <span class="saved-themes-empty-icon">◇</span>
          <p>No saved themes yet.</p>
          <p class="saved-themes-empty-hint">Use the "💾 Save Theme" button to save your current color configuration.</p>
        </div>
      ) : (
        <div class="saved-themes-list">
          {themes.map((t) => <SavedThemeCard key={t.id} theme={t} />)}
        </div>
      )}
    </div>
  );
}

// ── ImportThemeButton ─────────────────────────────────────────────────────────

function ImportThemeButton() {
  const store = useStore();
  const fileRef = useRef<HTMLInputElement>(null);
  const [error, setError] = useState<string | null>(null);

  async function handleFile(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    const err = await store.importTheme(file);
    setError(err ?? null);
    input.value = '';
  }

  return (
    <>
      <input
        ref={fileRef}
        type="file"
        accept=".json,application/json"
        style={{ display: 'none' }}
        onChange={(e) => void handleFile(e)}
      />
      <button class="btn btn-secondary" onClick={() => fileRef.current?.click()} title="Load a theme from a .json file">
        📂 Import
      </button>
      {error && <span class="theme-import-error">{error}</span>}
    </>
  );
}

// ── ThemeSettingsImpl ─────────────────────────────────────────────────────────

function ThemeSettingsImpl() {
  const store = useStore();
  const fx = store.effectSettings.value;

  return (
    <div class="theme-settings-layout">
      <div class="theme-settings">
        <div class="theme-settings-header">
          <h2 class="theme-settings-title">Color Theme Settings</h2>
          <p class="theme-settings-subtitle">
            Customize every color in the palette. Changes are applied instantly and saved to your browser.
          </p>
          <div class="theme-settings-actions">
            <button class="btn btn-primary" onClick={store.resetTheme}>Reset to Default</button>
            {store.randomizeTheme && (
              <button class="btn btn-primary" onClick={store.randomizeTheme}>🎲 Randomize</button>
            )}
            <SaveThemeButton />
            <button class="btn btn-secondary" onClick={() => store.exportTheme()} title="Export current theme as .json">
              📤 Export
            </button>
            <ImportThemeButton />
          </div>
        </div>

        {/* ── Presets ── */}
        <Section title="Theme Presets" icon="◆" defaultOpen={true}>
          <div class="theme-presets-grid">
            {store.presets.map((preset: ThemePreset) => (
              <button key={preset.name} class="theme-preset-card" onClick={() => store.applyPreset(preset)}>
                <div class="theme-preset-swatches">
                  <span class="theme-preset-swatch" style={{ background: preset.colors.bgPrimary }} />
                  <span class="theme-preset-swatch" style={{ background: preset.colors.accentOrange }} />
                  <span class="theme-preset-swatch" style={{ background: preset.colors.accentBlue }} />
                  <span class="theme-preset-swatch" style={{ background: preset.colors.levelError }} />
                  <span class="theme-preset-swatch" style={{ background: preset.colors.cinderEmber }} />
                </div>
                <div class="theme-preset-info">
                  <strong>{preset.name}</strong>
                  <span>{preset.description}</span>
                </div>
              </button>
            ))}
          </div>
        </Section>

        {/* ── Backgrounds ── */}
        <Section title="Backgrounds" icon="▧">
          <ColorRow label="Primary" description="Main app background" colorKey="bgPrimary" />
          <ColorRow label="Secondary" description="Header, panels" colorKey="bgSecondary" />
          <ColorRow label="Tertiary" description="Inputs, nested areas" colorKey="bgTertiary" />
          <ColorRow label="Hover" description="Hovered elements" colorKey="bgHover" />
          <ColorRow label="Active" description="Active/pressed state" colorKey="bgActive" />
        </Section>

        {/* ── Text / Fonts ── */}
        <Section title="Text & Fonts" icon="A">
          <ColorRow label="Primary Text" description="Main content text" colorKey="textPrimary" />
          <ColorRow label="Secondary Text" description="Labels, metadata" colorKey="textSecondary" />
          <ColorRow label="Muted Text" description="Disabled, hints" colorKey="textMuted" />
        </Section>

        {/* ── Borders ── */}
        <Section title="Borders" icon="□">
          <ColorRow label="Border" description="Panel and input borders" colorKey="borderColor" />
          <ColorRow label="Subtle Border" description="Very faint separators" colorKey="borderSubtle" />
        </Section>

        {/* ── Accents ── */}
        <Section title="Accent Colors" icon="◈">
          <ColorRow label="Blue" description="Links, focus rings" colorKey="accentBlue" />
          <ColorRow label="Green" description="Success, vine" colorKey="accentGreen" />
          <ColorRow label="Orange" description="Primary accent, bonfire" colorKey="accentOrange" />
          <ColorRow label="Purple" description="Special highlights" colorKey="accentPurple" />
          <ColorRow label="Yellow" description="Tarnished gold" colorKey="accentYellow" />
        </Section>

        {/* ── Log Levels ── */}
        <Section title="Log Level Colors" icon="▤">
          <ColorRow label="TRACE" description="Faintest level" colorKey="levelTrace" />
          <ColorRow label="DEBUG" description="Debug output" colorKey="levelDebug" />
          <ColorRow label="INFO" description="Informational" colorKey="levelInfo" />
          <ColorRow label="WARN" description="Warnings" colorKey="levelWarn" />
          <ColorRow label="ERROR" description="Errors" colorKey="levelError" />
        </Section>

        {/* ── Log Level Badge Text ── */}
        <Section title="Log Level Text Colors" icon="T">
          <p class="theme-section-hint">Text colors for log level badges.</p>
          <ColorRow label="TRACE Text" colorKey="levelTraceText" />
          <ColorRow label="DEBUG Text" colorKey="levelDebugText" />
          <ColorRow label="INFO Text" colorKey="levelInfoText" />
          <ColorRow label="WARN Text" colorKey="levelWarnText" />
          <ColorRow label="ERROR Text" colorKey="levelErrorText" />
        </Section>

        {/* ── Span Badge Colors ── */}
        <Section title="Span Badge Colors" icon="→">
          <ColorRow label="Enter Span" colorKey="spanEnterText" />
          <ColorRow label="Exit Span" colorKey="spanExitText" />
        </Section>

        {/* ── GPU Rendering ── */}
        <Section title="GPU Rendering" icon="⬢">
          <div class="theme-toggle-row">
            <div class="theme-color-info">
              <span class="theme-color-label">Enable GPU</span>
              <span class="theme-color-desc">Master switch for WebGPU rendering</span>
            </div>
            <label class="theme-toggle">
              <input
                type="checkbox"
                checked={gpuOverlayEnabled.value}
                onChange={(e) => { gpuOverlayEnabled.value = (e.target as HTMLInputElement).checked; }}
              />
              <span class="theme-toggle-slider" />
            </label>
          </div>
        </Section>

        {/* ── Sparks ── */}
        <Section title="Particles: Metal Sparks" icon="✦" className="effect-preview-sparks">
          <p class="theme-section-hint">Sparks spawn at the mouse cursor when hovering over elements.</p>
          <div class="theme-toggle-row">
            <div class="theme-color-info">
              <span class="theme-color-label">Enable Sparks</span>
            </div>
            <label class="toggle-switch">
              <input type="checkbox" checked={fx.sparksEnabled}
                onChange={(e) => store.updateEffect('sparksEnabled', (e.target as HTMLInputElement).checked)} />
              <span class="toggle-slider" />
            </label>
          </div>
          {fx.sparksEnabled && (<>
            <ColorRow label="Hot Core" colorKey="particleSparkCore" />
            <ColorRow label="Ember" colorKey="particleSparkEmber" />
            <ColorRow label="Steel" colorKey="particleSparkSteel" />
            {([
              { key: 'sparkSpeed' as const, label: 'Speed', max: 300 },
              { key: 'sparkCount' as const, label: 'Count', max: 200 },
              { key: 'sparkSize' as const, label: 'Size', max: 300 },
            ] as const).map(({ key, label, max }) => (
              <div class="theme-slider-row" key={key}>
                <div class="theme-color-info"><span class="theme-color-label">{label}</span></div>
                <div class="theme-slider-controls">
                  <input type="range" min="0" max={String(max)} step="1" value={fx[key]}
                    onInput={(e) => store.updateEffect(key, parseInt((e.target as HTMLInputElement).value, 10))}
                    class="theme-range-slider" />
                  <span class="theme-slider-value">{fx[key]}%</span>
                </div>
              </div>
            ))}
          </>)}
        </Section>

        {/* ── Embers ── */}
        <Section title="Particles: Embers / Ash" icon="🔥" className="effect-preview-embers">
          <p class="theme-section-hint">Rising embers/ash from hovered element borders.</p>
          <div class="theme-toggle-row">
            <div class="theme-color-info"><span class="theme-color-label">Enable Embers</span></div>
            <label class="toggle-switch">
              <input type="checkbox" checked={fx.embersEnabled}
                onChange={(e) => store.updateEffect('embersEnabled', (e.target as HTMLInputElement).checked)} />
              <span class="toggle-slider" />
            </label>
          </div>
          {fx.embersEnabled && (<>
            <ColorRow label="Hot" colorKey="particleEmberHot" />
            <ColorRow label="Base" colorKey="particleEmberBase" />
            {([
              { key: 'emberSpeed' as const, label: 'Speed', max: 300 },
              { key: 'emberCount' as const, label: 'Count', max: 200 },
              { key: 'emberSize' as const, label: 'Size', max: 300 },
            ] as const).map(({ key, label, max }) => (
              <div class="theme-slider-row" key={key}>
                <div class="theme-color-info"><span class="theme-color-label">{label}</span></div>
                <div class="theme-slider-controls">
                  <input type="range" min="0" max={String(max)} step="1" value={fx[key]}
                    onInput={(e) => store.updateEffect(key, parseInt((e.target as HTMLInputElement).value, 10))}
                    class="theme-range-slider" />
                  <span class="theme-slider-value">{fx[key]}%</span>
                </div>
              </div>
            ))}
          </>)}
        </Section>

        {/* ── Beams ── */}
        <Section title="Particles: Angelic Beams" icon="✧" className="effect-preview-beams">
          <p class="theme-section-hint">Pixel-thin vertical rays rising from the selected element.</p>
          <div class="theme-toggle-row">
            <div class="theme-color-info"><span class="theme-color-label">Enable Beams</span></div>
            <label class="toggle-switch">
              <input type="checkbox" checked={fx.beamsEnabled}
                onChange={(e) => store.updateEffect('beamsEnabled', (e.target as HTMLInputElement).checked)} />
              <span class="toggle-slider" />
            </label>
          </div>
          {fx.beamsEnabled && (<>
            <ColorRow label="Center" colorKey="particleBeamCenter" />
            <ColorRow label="Edge" colorKey="particleBeamEdge" />
            {([
              { key: 'beamSpeed' as const, label: 'Speed', max: 300 },
              { key: 'beamHeight' as const, label: 'Height', max: 100 },
              { key: 'beamCount' as const, label: 'Count', max: 1024 },
              { key: 'beamDrift' as const, label: 'Drift', max: 300 },
            ] as const).map(({ key, label, max }) => (
              <div class="theme-slider-row" key={key}>
                <div class="theme-color-info"><span class="theme-color-label">{label}</span></div>
                <div class="theme-slider-controls">
                  <input type="range" min={key === 'beamHeight' ? '10' : '0'} max={String(max)} step="1" value={fx[key]}
                    onInput={(e) => store.updateEffect(key, parseInt((e.target as HTMLInputElement).value, 10))}
                    class="theme-range-slider" />
                  <span class="theme-slider-value">{fx[key] || (key === 'beamCount' ? 'All' : '0')}</span>
                </div>
              </div>
            ))}
          </>)}
        </Section>

        {/* ── Glitter ── */}
        <Section title="Particles: Glitter" icon="✨" className="effect-preview-glitter">
          <p class="theme-section-hint">Twinkling sparkles drifting along hovered element borders.</p>
          <div class="theme-toggle-row">
            <div class="theme-color-info"><span class="theme-color-label">Enable Glitter</span></div>
            <label class="toggle-switch">
              <input type="checkbox" checked={fx.glitterEnabled}
                onChange={(e) => store.updateEffect('glitterEnabled', (e.target as HTMLInputElement).checked)} />
              <span class="toggle-slider" />
            </label>
          </div>
          {fx.glitterEnabled && (<>
            <ColorRow label="Warm" colorKey="particleGlitterWarm" />
            <ColorRow label="Cool" colorKey="particleGlitterCool" />
            {([
              { key: 'glitterSpeed' as const, label: 'Speed', max: 300 },
              { key: 'glitterCount' as const, label: 'Count', max: 200 },
              { key: 'glitterSize' as const, label: 'Size', max: 300 },
            ] as const).map(({ key, label, max }) => (
              <div class="theme-slider-row" key={key}>
                <div class="theme-color-info"><span class="theme-color-label">{label}</span></div>
                <div class="theme-slider-controls">
                  <input type="range" min="0" max={String(max)} step="1" value={fx[key]}
                    onInput={(e) => store.updateEffect(key, parseInt((e.target as HTMLInputElement).value, 10))}
                    class="theme-range-slider" />
                  <span class="theme-slider-value">{fx[key]}%</span>
                </div>
              </div>
            ))}
          </>)}
        </Section>

        {/* ── Cinder ── */}
        <Section title="Cinder Palette" icon="◎">
          <p class="theme-section-hint">The four-color cycle used for border glows and hover effects.</p>
          <div class="theme-toggle-row">
            <div class="theme-color-info"><span class="theme-color-label">Enable Cinder</span></div>
            <label class="toggle-switch">
              <input type="checkbox" checked={fx.cinderEnabled}
                onChange={(e) => store.updateEffect('cinderEnabled', (e.target as HTMLInputElement).checked)} />
              <span class="toggle-slider" />
            </label>
          </div>
          {fx.cinderEnabled && (<>
            <ColorRow label="Ember" colorKey="cinderEmber" />
            <ColorRow label="Gold" colorKey="cinderGold" />
            <ColorRow label="Ash" colorKey="cinderAsh" />
            <ColorRow label="Vine" colorKey="cinderVine" />
            <div class="theme-slider-row">
              <div class="theme-color-info"><span class="theme-color-label">Size</span></div>
              <div class="theme-slider-controls">
                <input type="range" min="0" max="300" step="1" value={fx.cinderSize}
                  onInput={(e) => store.updateEffect('cinderSize', parseInt((e.target as HTMLInputElement).value, 10))}
                  class="theme-range-slider" />
                <span class="theme-slider-value">{fx.cinderSize}%</span>
              </div>
            </div>
          </>)}
        </Section>

        {/* ── Smoke ── */}
        <Section title="Background Smoke" icon="☁">
          <p class="theme-section-hint">Base tones and noise parameters for the animated smoky background layers.</p>
          <div class="theme-toggle-row">
            <div class="theme-color-info"><span class="theme-color-label">Enable Smoke</span></div>
            <label class="toggle-switch">
              <input type="checkbox" checked={fx.smokeEnabled}
                onChange={(e) => store.updateEffect('smokeEnabled', (e.target as HTMLInputElement).checked)} />
              <span class="toggle-slider" />
            </label>
          </div>
          {fx.smokeEnabled && (<>
            <ColorRow label="Cool Tone" colorKey="smokeCool" />
            <ColorRow label="Warm Tone" colorKey="smokeWarm" />
            <ColorRow label="Moss Tone" colorKey="smokeMoss" />
            {([
              { key: 'smokeIntensity' as const, label: 'Intensity', max: 100 },
              { key: 'smokeSpeed' as const, label: 'Speed', max: 500 },
              { key: 'smokeWarmScale' as const, label: 'Warm Scale', max: 200 },
              { key: 'smokeCoolScale' as const, label: 'Cool Scale', max: 200 },
              { key: 'smokeMossScale' as const, label: 'Moss Scale', max: 200 },
              { key: 'grainIntensity' as const, label: 'Grain Intensity', max: 100 },
              { key: 'grainCoarseness' as const, label: 'Grain Coarseness', max: 100 },
              { key: 'grainSize' as const, label: 'Grain Size', max: 100 },
              { key: 'vignetteStrength' as const, label: 'Vignette', max: 100 },
              { key: 'underglowStrength' as const, label: 'Underglow', max: 100 },
            ] as const).map(({ key, label, max }) => (
              <div class="theme-slider-row" key={key}>
                <div class="theme-color-info"><span class="theme-color-label">{label}</span></div>
                <div class="theme-slider-controls">
                  <input type="range" min="0" max={String(max)} step="1" value={fx[key]}
                    onInput={(e) => store.updateEffect(key, parseInt((e.target as HTMLInputElement).value, 10))}
                    class="theme-range-slider" />
                  <span class="theme-slider-value">{fx[key]}{max === 100 ? '%' : ''}</span>
                </div>
              </div>
            ))}
          </>)}
        </Section>

        {/* ── Glass ── */}
        <Section title="Glass Panels" icon="◻" defaultOpen={true}>
          <p class="theme-section-hint">Sidebar, header, and tab-bar glass panel opacity and blur.</p>
          {([
            { key: 'glassOpacity' as const, label: 'Opacity', desc: 'Background panel transparency' },
            { key: 'glassBlur' as const, label: 'Blur', desc: 'Backdrop blur intensity' },
          ] as const).map(({ key, label, desc }) => (
            <div class="theme-slider-row" key={key}>
              <div class="theme-color-info">
                <span class="theme-color-label">{label}</span>
                <span class="theme-color-desc">{desc}</span>
              </div>
              <div class="theme-slider-controls">
                <input type="range" min="0" max="100" step="1" value={fx[key]}
                  onInput={(e) => store.updateEffect(key, parseInt((e.target as HTMLInputElement).value, 10))}
                  class="theme-range-slider" />
                <span class="theme-slider-value">{fx[key]}%</span>
              </div>
            </div>
          ))}
        </Section>

        {/* ── CRT ── */}
        <Section title="CRT Effect" icon="▤" defaultOpen={true}>
          <p class="theme-section-hint">Retro CRT post-processing — scanlines, pixel grid, edge shadow, torch flicker.</p>
          <div class="theme-toggle-row">
            <div class="theme-color-info"><span class="theme-color-label">Enable CRT</span></div>
            <label class="toggle-switch">
              <input type="checkbox" checked={fx.crtEnabled}
                onChange={(e) => store.updateEffect('crtEnabled', (e.target as HTMLInputElement).checked)} />
              <span class="toggle-slider" />
            </label>
          </div>
          {fx.crtEnabled && (<>
            {([
              { key: 'crtScanlinesH' as const, label: 'H Scanlines' },
              { key: 'crtScanlinesV' as const, label: 'V Scanlines' },
              { key: 'crtEdgeShadow' as const, label: 'Edge Shadow' },
              { key: 'crtFlicker' as const, label: 'Flicker' },
              { key: 'crtLineWidth' as const, label: 'Line Width' },
            ] as const).map(({ key, label }) => (
              <div class="theme-slider-row" key={key}>
                <div class="theme-color-info"><span class="theme-color-label">{label}</span></div>
                <div class="theme-slider-controls">
                  <input type="range" min="0" max="100" step="1" value={fx[key]}
                    onInput={(e) => store.updateEffect(key, parseInt((e.target as HTMLInputElement).value, 10))}
                    class="theme-range-slider" />
                  <span class="theme-slider-value">{fx[key]}%</span>
                </div>
              </div>
            ))}
            <div class="theme-slider-row">
              <div class="theme-color-info"><span class="theme-color-label">Scanline Color</span></div>
              <input
                type="color"
                value={`#${(fx.crtColor ?? [100, 80, 60]).map((c: number) => c.toString(16).padStart(2, '0')).join('')}`}
                onInput={(e) => {
                  const hex = (e.target as HTMLInputElement).value;
                  const r = parseInt(hex.slice(1, 3), 16);
                  const g = parseInt(hex.slice(3, 5), 16);
                  const b = parseInt(hex.slice(5, 7), 16);
                  store.updateEffect('crtColor', [r, g, b]);
                }}
                class="theme-color-picker"
              />
            </div>
          </>)}
        </Section>
      </div>

      <SavedThemesPanel />
    </div>
  );
}

// ── Public export ─────────────────────────────────────────────────────────────

export { type ThemeSettingsStore };

/**
 * Shared ColorTheme settings component.
 *
 * Each viewer creates a `ThemeSettingsStore` from its own signals and functions
 * and passes it here as a prop.
 */
export function ThemeSettings({ store }: { store: ThemeSettingsStore }) {
  return (
    <Ctx.Provider value={store}>
      <ThemeSettingsImpl />
    </Ctx.Provider>
  );
}

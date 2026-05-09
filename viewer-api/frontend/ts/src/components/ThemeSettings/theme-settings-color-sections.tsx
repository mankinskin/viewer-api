import type { ThemePreset } from '../../store/theme';
import { gpuOverlayEnabled } from '../WgpuOverlay/WgpuOverlay';
import { useThemeSettingsStore } from './theme-settings-context';
import { ColorRow, Section } from './theme-settings-ui';

export function ThemeColorSections() {
  const store = useThemeSettingsStore();

  return (
    <>
      <Section title="Theme Presets" icon="◆" defaultOpen={true}>
        <div class="theme-presets-grid">
          {store.presets.map((preset: ThemePreset) => (
            <button
              key={preset.name}
              class="theme-preset-card"
              onClick={() => store.applyPreset(preset)}
            >
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

      <Section title="Backgrounds" icon="▧">
        <ColorRow label="Primary" description="Main app background" colorKey="bgPrimary" />
        <ColorRow label="Secondary" description="Header, panels" colorKey="bgSecondary" />
        <ColorRow label="Tertiary" description="Inputs, nested areas" colorKey="bgTertiary" />
        <ColorRow label="Hover" description="Hovered elements" colorKey="bgHover" />
        <ColorRow label="Active" description="Active/pressed state" colorKey="bgActive" />
      </Section>

      <Section title="Text & Fonts" icon="A">
        <ColorRow label="Primary Text" description="Main content text" colorKey="textPrimary" />
        <ColorRow label="Secondary Text" description="Labels, metadata" colorKey="textSecondary" />
        <ColorRow label="Muted Text" description="Disabled, hints" colorKey="textMuted" />
      </Section>

      <Section title="Borders" icon="□">
        <ColorRow label="Border" description="Panel and input borders" colorKey="borderColor" />
        <ColorRow label="Subtle Border" description="Very faint separators" colorKey="borderSubtle" />
      </Section>

      <Section title="Accent Colors" icon="◈">
        <ColorRow label="Blue" description="Links, focus rings" colorKey="accentBlue" />
        <ColorRow label="Green" description="Success, vine" colorKey="accentGreen" />
        <ColorRow label="Orange" description="Primary accent, bonfire" colorKey="accentOrange" />
        <ColorRow label="Purple" description="Special highlights" colorKey="accentPurple" />
        <ColorRow label="Yellow" description="Tarnished gold" colorKey="accentYellow" />
      </Section>

      <Section title="Log Level Colors" icon="▤">
        <ColorRow label="TRACE" description="Faintest level" colorKey="levelTrace" />
        <ColorRow label="DEBUG" description="Debug output" colorKey="levelDebug" />
        <ColorRow label="INFO" description="Informational" colorKey="levelInfo" />
        <ColorRow label="WARN" description="Warnings" colorKey="levelWarn" />
        <ColorRow label="ERROR" description="Errors" colorKey="levelError" />
      </Section>

      <Section title="Log Level Text Colors" icon="T">
        <p class="theme-section-hint">Text colors for log level badges.</p>
        <ColorRow label="TRACE Text" colorKey="levelTraceText" />
        <ColorRow label="DEBUG Text" colorKey="levelDebugText" />
        <ColorRow label="INFO Text" colorKey="levelInfoText" />
        <ColorRow label="WARN Text" colorKey="levelWarnText" />
        <ColorRow label="ERROR Text" colorKey="levelErrorText" />
      </Section>

      <Section title="Span Badge Colors" icon="→">
        <ColorRow label="Enter Span" colorKey="spanEnterText" />
        <ColorRow label="Exit Span" colorKey="spanExitText" />
      </Section>

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
              onChange={(event) => {
                gpuOverlayEnabled.value = (event.target as HTMLInputElement).checked;
              }}
            />
            <span class="theme-toggle-slider" />
          </label>
        </div>
      </Section>
    </>
  );
}
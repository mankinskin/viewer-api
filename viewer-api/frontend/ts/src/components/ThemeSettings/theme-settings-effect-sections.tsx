import { useThemeSettingsStore } from './theme-settings-context';
import { ColorRow, Section } from './theme-settings-ui';

export function ThemeEffectSections() {
  const store = useThemeSettingsStore();
  const fx = store.effectSettings.value;

  return (
    <>
      <Section title="Particles: Metal Sparks" icon="✦" className="effect-preview-sparks">
        <p class="theme-section-hint">Sparks spawn at the mouse cursor when hovering over elements.</p>
        <div class="theme-toggle-row">
          <div class="theme-color-info">
            <span class="theme-color-label">Enable Sparks</span>
          </div>
          <label class="toggle-switch">
            <input
              type="checkbox"
              checked={fx.sparksEnabled}
              onChange={(event) =>
                store.updateEffect('sparksEnabled', (event.target as HTMLInputElement).checked)
              }
            />
            <span class="toggle-slider" />
          </label>
        </div>
        {fx.sparksEnabled && (
          <>
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
                  <input
                    type="range"
                    min="0"
                    max={String(max)}
                    step="1"
                    value={fx[key]}
                    onInput={(event) =>
                      store.updateEffect(key, parseInt((event.target as HTMLInputElement).value, 10))
                    }
                    class="theme-range-slider"
                  />
                  <span class="theme-slider-value">{fx[key]}%</span>
                </div>
              </div>
            ))}
          </>
        )}
      </Section>

      <Section title="Particles: Embers / Ash" icon="🔥" className="effect-preview-embers">
        <p class="theme-section-hint">Rising embers/ash from hovered element borders.</p>
        <div class="theme-toggle-row">
          <div class="theme-color-info"><span class="theme-color-label">Enable Embers</span></div>
          <label class="toggle-switch">
            <input
              type="checkbox"
              checked={fx.embersEnabled}
              onChange={(event) =>
                store.updateEffect('embersEnabled', (event.target as HTMLInputElement).checked)
              }
            />
            <span class="toggle-slider" />
          </label>
        </div>
        {fx.embersEnabled && (
          <>
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
                  <input
                    type="range"
                    min="0"
                    max={String(max)}
                    step="1"
                    value={fx[key]}
                    onInput={(event) =>
                      store.updateEffect(key, parseInt((event.target as HTMLInputElement).value, 10))
                    }
                    class="theme-range-slider"
                  />
                  <span class="theme-slider-value">{fx[key]}%</span>
                </div>
              </div>
            ))}
          </>
        )}
      </Section>

      <Section title="Particles: Angelic Beams" icon="✧" className="effect-preview-beams">
        <p class="theme-section-hint">Pixel-thin vertical rays rising from the selected element.</p>
        <div class="theme-toggle-row">
          <div class="theme-color-info"><span class="theme-color-label">Enable Beams</span></div>
          <label class="toggle-switch">
            <input
              type="checkbox"
              checked={fx.beamsEnabled}
              onChange={(event) =>
                store.updateEffect('beamsEnabled', (event.target as HTMLInputElement).checked)
              }
            />
            <span class="toggle-slider" />
          </label>
        </div>
        {fx.beamsEnabled && (
          <>
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
                  <input
                    type="range"
                    min={key === 'beamHeight' ? '10' : '0'}
                    max={String(max)}
                    step="1"
                    value={fx[key]}
                    onInput={(event) =>
                      store.updateEffect(key, parseInt((event.target as HTMLInputElement).value, 10))
                    }
                    class="theme-range-slider"
                  />
                  <span class="theme-slider-value">
                    {fx[key] || (key === 'beamCount' ? 'All' : '0')}
                  </span>
                </div>
              </div>
            ))}
          </>
        )}
      </Section>

      <Section title="Particles: Glitter" icon="✨" className="effect-preview-glitter">
        <p class="theme-section-hint">Twinkling sparkles drifting along hovered element borders.</p>
        <div class="theme-toggle-row">
          <div class="theme-color-info"><span class="theme-color-label">Enable Glitter</span></div>
          <label class="toggle-switch">
            <input
              type="checkbox"
              checked={fx.glitterEnabled}
              onChange={(event) =>
                store.updateEffect('glitterEnabled', (event.target as HTMLInputElement).checked)
              }
            />
            <span class="toggle-slider" />
          </label>
        </div>
        {fx.glitterEnabled && (
          <>
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
                  <input
                    type="range"
                    min="0"
                    max={String(max)}
                    step="1"
                    value={fx[key]}
                    onInput={(event) =>
                      store.updateEffect(key, parseInt((event.target as HTMLInputElement).value, 10))
                    }
                    class="theme-range-slider"
                  />
                  <span class="theme-slider-value">{fx[key]}%</span>
                </div>
              </div>
            ))}
          </>
        )}
      </Section>

      <Section title="Cinder Palette" icon="◎">
        <p class="theme-section-hint">The four-color cycle used for border glows and hover effects.</p>
        <div class="theme-toggle-row">
          <div class="theme-color-info"><span class="theme-color-label">Enable Cinder</span></div>
          <label class="toggle-switch">
            <input
              type="checkbox"
              checked={fx.cinderEnabled}
              onChange={(event) =>
                store.updateEffect('cinderEnabled', (event.target as HTMLInputElement).checked)
              }
            />
            <span class="toggle-slider" />
          </label>
        </div>
        {fx.cinderEnabled && (
          <>
            <ColorRow label="Ember" colorKey="cinderEmber" />
            <ColorRow label="Gold" colorKey="cinderGold" />
            <ColorRow label="Ash" colorKey="cinderAsh" />
            <ColorRow label="Vine" colorKey="cinderVine" />
            <div class="theme-slider-row">
              <div class="theme-color-info"><span class="theme-color-label">Size</span></div>
              <div class="theme-slider-controls">
                <input
                  type="range"
                  min="0"
                  max="300"
                  step="1"
                  value={fx.cinderSize}
                  onInput={(event) =>
                    store.updateEffect('cinderSize', parseInt((event.target as HTMLInputElement).value, 10))
                  }
                  class="theme-range-slider"
                />
                <span class="theme-slider-value">{fx.cinderSize}%</span>
              </div>
            </div>
          </>
        )}
      </Section>

      <Section title="Background Smoke" icon="☁">
        <p class="theme-section-hint">Base tones and noise parameters for the animated smoky background layers.</p>
        <div class="theme-toggle-row">
          <div class="theme-color-info"><span class="theme-color-label">Enable Smoke</span></div>
          <label class="toggle-switch">
            <input
              type="checkbox"
              checked={fx.smokeEnabled}
              onChange={(event) =>
                store.updateEffect('smokeEnabled', (event.target as HTMLInputElement).checked)
              }
            />
            <span class="toggle-slider" />
          </label>
        </div>
        {fx.smokeEnabled && (
          <>
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
                  <input
                    type="range"
                    min="0"
                    max={String(max)}
                    step="1"
                    value={fx[key]}
                    onInput={(event) =>
                      store.updateEffect(key, parseInt((event.target as HTMLInputElement).value, 10))
                    }
                    class="theme-range-slider"
                  />
                  <span class="theme-slider-value">{fx[key]}{max === 100 ? '%' : ''}</span>
                </div>
              </div>
            ))}
          </>
        )}
      </Section>

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
              <input
                type="range"
                min="0"
                max="100"
                step="1"
                value={fx[key]}
                onInput={(event) =>
                  store.updateEffect(key, parseInt((event.target as HTMLInputElement).value, 10))
                }
                class="theme-range-slider"
              />
              <span class="theme-slider-value">{fx[key]}%</span>
            </div>
          </div>
        ))}
      </Section>

      <Section title="CRT Effect" icon="▤" defaultOpen={true}>
        <p class="theme-section-hint">Retro CRT post-processing - scanlines, pixel grid, edge shadow, torch flicker.</p>
        <div class="theme-toggle-row">
          <div class="theme-color-info"><span class="theme-color-label">Enable CRT</span></div>
          <label class="toggle-switch">
            <input
              type="checkbox"
              checked={fx.crtEnabled}
              onChange={(event) =>
                store.updateEffect('crtEnabled', (event.target as HTMLInputElement).checked)
              }
            />
            <span class="toggle-slider" />
          </label>
        </div>
        {fx.crtEnabled && (
          <>
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
                  <input
                    type="range"
                    min="0"
                    max="100"
                    step="1"
                    value={fx[key]}
                    onInput={(event) =>
                      store.updateEffect(key, parseInt((event.target as HTMLInputElement).value, 10))
                    }
                    class="theme-range-slider"
                  />
                  <span class="theme-slider-value">{fx[key]}%</span>
                </div>
              </div>
            ))}
            <div class="theme-slider-row">
              <div class="theme-color-info"><span class="theme-color-label">Scanline Color</span></div>
              <input
                type="color"
                value={`#${(fx.crtColor ?? [100, 80, 60])
                  .map((channel: number) => channel.toString(16).padStart(2, '0'))
                  .join('')}`}
                onInput={(event) => {
                  const hex = (event.target as HTMLInputElement).value;
                  const r = parseInt(hex.slice(1, 3), 16);
                  const g = parseInt(hex.slice(3, 5), 16);
                  const b = parseInt(hex.slice(5, 7), 16);
                  store.updateEffect('crtColor', [r, g, b]);
                }}
                class="theme-color-picker"
              />
            </div>
          </>
        )}
      </Section>
    </>
  );
}
// GPU particle and background effect configuration shared across viewer tools.

export interface EffectSettings {
  /** Glass panel background opacity 0-100 (maps to 0.0-0.4 alpha). */
  glassOpacity: number;
  /** Glass panel backdrop blur 0-100 (maps to 0-16px). */
  glassBlur: number;
  crtEnabled: boolean;
  /** Horizontal scanlines (+ pixel grid) intensity 0-100. */
  crtScanlinesH: number;
  /** Vertical scanlines (+ pixel grid) intensity 0-100. */
  crtScanlinesV: number;
  /** Edge/border shadow intensity 0-100. */
  crtEdgeShadow: number;
  /** Torch flicker intensity 0-100. */
  crtFlicker: number;
  /** Scanline width/thickness 0-100. */
  crtLineWidth: number;
  /** Scanline tint color [R, G, B] 0-255. */
  crtColor: [number, number, number];
  smokeEnabled: boolean;
  smokeIntensity: number;
  smokeSpeed: number;
  smokeWarmScale: number;
  smokeCoolScale: number;
  smokeMossScale: number;
  grainIntensity: number;
  grainCoarseness: number;
  grainSize: number;
  vignetteStrength: number;
  underglowStrength: number;
  sparkSpeed: number;
  sparksEnabled: boolean;
  emberSpeed: number;
  embersEnabled: boolean;
  beamSpeed: number;
  beamsEnabled: boolean;
  glitterSpeed: number;
  glitterEnabled: boolean;
  beamHeight: number;
  beamDrift: number;
  beamCount: number;
  sparkCount: number;
  sparkSize: number;
  emberCount: number;
  emberSize: number;
  glitterCount: number;
  glitterSize: number;
  cinderSize: number;
  cinderEnabled: boolean;
}

export const DEFAULT_EFFECT_SETTINGS: EffectSettings = {
  glassOpacity: 35,
  glassBlur: 25,
  crtEnabled: true,
  crtScanlinesH: 20,
  crtScanlinesV: 12,
  crtEdgeShadow: 35,
  crtFlicker: 12,
  crtLineWidth: 50,
  crtColor: [100, 80, 60] as [number, number, number],
  smokeEnabled: true,
  smokeIntensity: 40,
  smokeSpeed: 50,
  smokeWarmScale: 100,
  smokeCoolScale: 100,
  smokeMossScale: 100,
  grainIntensity: 20,
  grainCoarseness: 40,
  grainSize: 35,
  vignetteStrength: 40,
  underglowStrength: 25,
  sparkSpeed: 70,
  sparksEnabled: true,
  emberSpeed: 70,
  embersEnabled: true,
  beamSpeed: 50,
  beamsEnabled: true,
  glitterSpeed: 60,
  glitterEnabled: true,
  beamHeight: 35,
  beamDrift: 80,
  beamCount: 48,
  sparkCount: 40,
  sparkSize: 70,
  emberCount: 40,
  emberSize: 70,
  glitterCount: 40,
  glitterSize: 60,
  cinderSize: 70,
  cinderEnabled: true,
};

/**
 * All GPU effects disabled. Safe default for viewers that opt into effects only on demand.
 */
export const DEFAULT_EFFECT_SETTINGS_OFF: EffectSettings = {
  glassOpacity: 0,
  glassBlur: 0,
  crtEnabled: false,
  crtScanlinesH: 0,
  crtScanlinesV: 0,
  crtEdgeShadow: 0,
  crtFlicker: 0,
  crtLineWidth: 50,
  crtColor: [100, 80, 60] as [number, number, number],
  smokeEnabled: false,
  smokeIntensity: 0,
  smokeSpeed: 50,
  smokeWarmScale: 100,
  smokeCoolScale: 100,
  smokeMossScale: 100,
  grainIntensity: 0,
  grainCoarseness: 40,
  grainSize: 35,
  vignetteStrength: 0,
  underglowStrength: 0,
  sparkSpeed: 70,
  sparksEnabled: false,
  emberSpeed: 70,
  embersEnabled: false,
  beamSpeed: 50,
  beamsEnabled: false,
  glitterSpeed: 60,
  glitterEnabled: false,
  beamHeight: 35,
  beamDrift: 80,
  beamCount: 48,
  sparkCount: 40,
  sparkSize: 70,
  emberCount: 40,
  emberSize: 70,
  glitterCount: 40,
  glitterSize: 60,
  cinderSize: 70,
  cinderEnabled: false,
};
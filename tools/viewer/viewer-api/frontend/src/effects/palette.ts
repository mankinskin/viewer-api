// palette.ts — Build GPU palette buffer from ThemeColors
//
// Maps the theme store's hex color values into a Float32Array matching
// the ThemePalette struct layout in palette.wgsl.

import { hexToVec3, type ThemeColors } from '../store/theme';

/** Number of vec4f entries in the palette uniform buffer. */
export const PALETTE_VEC4_COUNT = 24;

/** Byte size of the palette uniform buffer (384 bytes). */
export const PALETTE_BYTE_SIZE = PALETTE_VEC4_COUNT * 16;

/**
 * Build a Float32Array palette from theme colors.
 * Layout matches the ThemePalette struct in palette.wgsl exactly.
 */
export function buildPaletteBuffer(colors: ThemeColors): Float32Array {
    const buf = new Float32Array(PALETTE_VEC4_COUNT * 4);  // 96 floats
    let i = 0;

    function writeVec4(hex: string, a: number = 1.0) {
        const [r, g, b] = hexToVec3(hex);
        buf[i++] = r; buf[i++] = g; buf[i++] = b; buf[i++] = a;
    }

    // Particle: Metal Spark [0-2]
    writeVec4(colors.particleSparkCore);     // [0]
    writeVec4(colors.particleSparkEmber);    // [1]
    writeVec4(colors.particleSparkSteel);    // [2]

    // Particle: Ember / Ash [3]
    writeVec4(colors.particleEmberHot);      // [3]

    // Particle: Angelic Beam [4-5]
    writeVec4(colors.particleBeamCenter);    // [4]
    writeVec4(colors.particleBeamEdge);      // [5]

    // Particle: Glitter [6-7]
    writeVec4(colors.particleGlitterWarm);   // [6]
    writeVec4(colors.particleGlitterCool);   // [7]

    // Cinder palette [8-11]
    writeVec4(colors.cinderEmber);           // [8]
    writeVec4(colors.cinderGold);            // [9]
    writeVec4(colors.cinderAsh);             // [10]
    writeVec4(colors.cinderVine);            // [11]

    // Background smoke tones [12-14]
    writeVec4(colors.smokeCool);             // [12]
    writeVec4(colors.smokeWarm);             // [13]
    writeVec4(colors.smokeMoss);             // [14]

    // Kind-specific glow colors [15-22]
    // Derived from existing theme colors with shader-appropriate intensities
    writeVec4(colors.borderColor);           // [15] kind_structural
    writeVec4(colors.levelError);            // [16] kind_error
    writeVec4(colors.levelWarn);             // [17] kind_warn
    writeVec4(colors.levelInfo);             // [18] kind_info
    writeVec4(colors.levelDebug);            // [19] kind_debug
    writeVec4(colors.accentGreen);           // [20] kind_span
    writeVec4(colors.accentOrange);          // [21] kind_selected
    writeVec4(colors.levelError);            // [22] kind_panic (same base as error, shader varies intensity)

    // Padding [23]
    buf[i++] = 0; buf[i++] = 0; buf[i++] = 0; buf[i++] = 0;

    return buf;
}

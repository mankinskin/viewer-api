import type { EffectSettings } from '../../store/theme';

import type { AppSchema } from './schemas';
import {
    consumeParticleVPDirty,
    getOverlayCameraPos,
    getOverlayParticleInvVP,
    getOverlayParticleVP,
    getOverlayParticleViewport,
    getOverlayRefDepth,
    getOverlayWorldScale,
} from './overlay-api';

export function packRenderUniforms(options: {
    uniforms: Float32Array;
    canvas: HTMLCanvasElement;
    schema: AppSchema;
    effects: EffectSettings;
    fxEnabled: boolean;
    count: number;
    mx: number;
    my: number;
    dt: number;
    time: number;
    hoverIdx: number;
    hoverStartTime: number;
    selectedIdx: number;
    scrollDelta: { dx: number; dy: number };
}): void {
    const {
        uniforms,
        canvas,
        schema,
        effects,
        fxEnabled,
        count,
        mx,
        my,
        dt,
        time,
        hoverIdx,
        hoverStartTime,
        selectedIdx,
        scrollDelta,
    } = options;

    uniforms[0] = time;
    uniforms[1] = canvas.width;
    uniforms[2] = canvas.height;
    uniforms[3] = fxEnabled ? count : 0;
    uniforms[4] = fxEnabled ? mx : -9999;
    uniforms[5] = fxEnabled ? my : -9999;
    uniforms[6] = dt;
    uniforms[7] = fxEnabled ? hoverIdx : -1;
    uniforms[8] = hoverStartTime;
    uniforms[9] = fxEnabled ? selectedIdx : -1;

    const crtOn = fxEnabled && effects.crtEnabled;
    uniforms[10] = crtOn ? effects.crtScanlinesH / 100 : 0;
    uniforms[11] = crtOn ? effects.crtScanlinesV / 100 : 0;
    uniforms[12] = crtOn ? effects.crtEdgeShadow / 100 : 0;
    uniforms[13] = crtOn ? effects.crtFlicker / 100 : 0;
    uniforms[14] = crtOn ? effects.crtLineWidth / 100 : 0;
    uniforms[15] = fxEnabled && effects.smokeEnabled ? effects.smokeIntensity / 100 : 0;
    uniforms[16] = fxEnabled && effects.smokeEnabled ? effects.smokeSpeed / 100 : 0;
    uniforms[17] = fxEnabled && effects.smokeEnabled ? effects.smokeWarmScale / 100 : 0;
    uniforms[18] = fxEnabled && effects.smokeEnabled ? effects.smokeCoolScale / 100 : 0;
    uniforms[19] = fxEnabled && effects.smokeEnabled ? effects.smokeMossScale / 100 : 0;
    uniforms[20] = fxEnabled ? effects.grainIntensity / 100 : 0;
    uniforms[21] = fxEnabled ? effects.grainCoarseness / 100 : 0;
    uniforms[22] = fxEnabled ? effects.grainSize / 100 : 0;
    uniforms[23] = fxEnabled ? effects.vignetteStrength / 100 : 0;
    uniforms[24] = fxEnabled ? effects.underglowStrength / 100 : 0;
    uniforms[25] = fxEnabled && effects.sparksEnabled ? effects.sparkSpeed / 100 : 0;
    uniforms[26] = fxEnabled && effects.embersEnabled ? effects.emberSpeed / 100 : 0;
    uniforms[27] = fxEnabled && effects.beamsEnabled ? effects.beamSpeed / 100 : 0;
    uniforms[28] = fxEnabled && effects.glitterEnabled ? effects.glitterSpeed / 100 : 0;
    uniforms[29] = fxEnabled ? effects.beamHeight : 0;
    uniforms[30] = fxEnabled ? effects.beamCount : 0;
    uniforms[31] = fxEnabled ? effects.beamDrift / 100 : 0;
    uniforms[32] = scrollDelta.dx;
    uniforms[33] = scrollDelta.dy;
    uniforms[34] = fxEnabled && effects.sparksEnabled ? effects.sparkCount / 100 : 0;
    uniforms[35] = fxEnabled && effects.sparksEnabled ? effects.sparkSize / 100 : 0;
    uniforms[36] = fxEnabled && effects.embersEnabled ? effects.emberCount / 100 : 0;
    uniforms[37] = fxEnabled && effects.embersEnabled ? effects.emberSize / 100 : 0;
    uniforms[38] = fxEnabled && effects.glitterEnabled ? effects.glitterCount / 100 : 0;
    uniforms[39] = fxEnabled && effects.glitterEnabled ? effects.glitterSize / 100 : 0;
    uniforms[40] = fxEnabled && effects.cinderEnabled ? effects.cinderSize / 100 : 0;

    const crtColor = effects.crtColor ?? [100, 80, 60];
    uniforms[48] = crtOn ? crtColor[0] / 255 : 0;
    uniforms[49] = crtOn ? crtColor[1] / 255 : 0;
    uniforms[50] = crtOn ? crtColor[2] / 255 : 0;
    uniforms[51] = 0;

    const is3DView = schema.isActive3DView();
    const width = canvas.width;
    const height = canvas.height;
    if (is3DView && consumeParticleVPDirty()) {
        const viewport = getOverlayParticleViewport();
        uniforms[41] = getOverlayRefDepth();
        uniforms[42] = getOverlayWorldScale();
        uniforms[43] = viewport[0];
        uniforms[44] = viewport[1];
        uniforms[45] = viewport[2];
        uniforms[46] = viewport[3];

        const cameraPos = getOverlayCameraPos();
        uniforms[52] = cameraPos[0];
        uniforms[53] = cameraPos[1];
        uniforms[54] = cameraPos[2];
        uniforms[55] = 0;
        uniforms.set(getOverlayParticleVP(), 56);
        uniforms.set(getOverlayParticleInvVP(), 72);
    } else if (!is3DView) {
        uniforms[41] = 0;
        uniforms[42] = 1;
        uniforms[43] = 0;
        uniforms[44] = 0;
        uniforms[45] = width;
        uniforms[46] = height;
        uniforms[52] = 0;
        uniforms[53] = 0;
        uniforms[54] = 0;
        uniforms[55] = 0;

        uniforms[56] = 2 / width;
        uniforms[57] = 0;
        uniforms[58] = 0;
        uniforms[59] = 0;
        uniforms[60] = 0;
        uniforms[61] = -2 / height;
        uniforms[62] = 0;
        uniforms[63] = 0;
        uniforms[64] = 0;
        uniforms[65] = 0;
        uniforms[66] = 1;
        uniforms[67] = 0;
        uniforms[68] = -1;
        uniforms[69] = 1;
        uniforms[70] = 0;
        uniforms[71] = 1;

        uniforms[72] = width / 2;
        uniforms[73] = 0;
        uniforms[74] = 0;
        uniforms[75] = 0;
        uniforms[76] = 0;
        uniforms[77] = -height / 2;
        uniforms[78] = 0;
        uniforms[79] = 0;
        uniforms[80] = 0;
        uniforms[81] = 0;
        uniforms[82] = 1;
        uniforms[83] = 0;
        uniforms[84] = width / 2;
        uniforms[85] = height / 2;
        uniforms[86] = 0;
        uniforms[87] = 1;
    }

    uniforms[47] = schema.getCurrentViewId();
}
// palette.wgsl — Shared ThemePalette struct definition
//
// Included by ALL shaders that need configurable theme colors.
// The binding declaration (e.g. @group(0) @binding(3)) is in each
// pipeline-specific shader file, NOT here.

struct ThemePalette {
    // Particle: Metal Spark [0-2]
    spark_core     : vec4f,   // hot white-yellow center
    spark_ember    : vec4f,   // outer ember glow
    spark_steel    : vec4f,   // metallic highlight

    // Particle: Ember / Ash [3]
    ember_hot      : vec4f,   // bright hot center

    // Particle: Angelic Beam [4-5]
    beam_center    : vec4f,   // golden-white core
    beam_edge      : vec4f,   // warm gold edge

    // Particle: Glitter [6-7]
    glitter_warm   : vec4f,   // golden-white base
    glitter_cool   : vec4f,   // blue-white variation

    // Cinder palette cycle [8-11]
    cinder_ember   : vec4f,   // deep orange-red
    cinder_gold    : vec4f,   // tarnished gold
    cinder_ash     : vec4f,   // cool grey
    cinder_vine    : vec4f,   // deep green

    // Background smoke tones [12-14]
    smoke_cool     : vec4f,   // blue-grey
    smoke_warm     : vec4f,   // brown-amber
    smoke_moss     : vec4f,   // mossy mid-tone

    // Kind-specific glow colors [15-22]
    kind_structural : vec4f,  // kind 0: structural UI border
    kind_error      : vec4f,  // kind 1: error log entry
    kind_warn       : vec4f,  // kind 2: warn log entry
    kind_info       : vec4f,  // kind 3: info log entry
    kind_debug      : vec4f,  // kind 4: debug/trace log entry
    kind_span       : vec4f,  // kind 5: span-highlighted
    kind_selected   : vec4f,  // kind 6: selected entry
    kind_panic      : vec4f,  // kind 7: panic entry

    _pad            : vec4f,  // padding to 384 bytes
}

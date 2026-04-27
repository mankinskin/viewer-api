//! Orbit camera + mouse interaction state.

/// Vertical FOV (radians) — 45°.
pub const CAMERA_FOV:  f32 = std::f32::consts::FRAC_PI_4;
pub const CAMERA_NEAR: f32 = 0.1;
pub const CAMERA_FAR:  f32 = 200.0;

/// viewProj(64) + eye(16) + time(16) = 96 bytes = 24 floats.
pub const CAM_UNIFORM_FLOATS: usize = 24;

/// ThemePalette: 24 × vec4f = 96 floats. (Unused by the edge shader, but
/// the pipeline declares the binding so the buffer must exist.)
pub const PALETTE_FLOATS: usize = 96;

#[derive(Debug, Clone)]
pub struct Camera {
    pub yaw:      f32,
    pub pitch:    f32,
    pub distance: f32,
    pub target:   [f32; 3],
}

impl Default for Camera {
    fn default() -> Self {
        Self { yaw: 0.3, pitch: 0.4, distance: 25.0, target: [0.0, 0.0, -4.0] }
    }
}

impl Camera {
    pub fn eye(&self) -> [f32; 3] {
        let cp = self.pitch.cos();
        [
            self.target[0] + self.distance * cp * self.yaw.sin(),
            self.target[1] + self.distance * self.pitch.sin(),
            self.target[2] + self.distance * cp * self.yaw.cos(),
        ]
    }

    /// Frame the camera so a sphere of radius `radius` around `centre` is
    /// fully visible.
    pub fn frame(&mut self, centre: [f32; 3], radius: f32) {
        self.target = centre;
        let half_fov_tan = (CAMERA_FOV * 0.5).tan();
        self.distance = ((radius / half_fov_tan) * 1.3).clamp(12.0, 120.0);
    }
}

#[derive(Debug, Clone, Default)]
pub struct MouseState {
    pub orbiting: bool,
    pub panning:  bool,
    pub last_x:   f64,
    pub last_y:   f64,
}

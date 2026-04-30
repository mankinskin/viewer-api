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

    /// Apply a `CameraCommand` to this camera, framing the layout bounds
    /// when the command needs them.
    pub fn apply_command(&mut self, cmd: &CameraCommand, bounds: ([f32; 3], f32)) {
        let (centre, radius) = bounds;
        match *cmd {
            CameraCommand::ResetToDefault => {
                let def = Camera::default();
                self.yaw = def.yaw;
                self.pitch = def.pitch;
                self.frame(centre, radius);
            }
            CameraCommand::ResetTo { yaw, pitch } => {
                self.yaw = yaw;
                self.pitch = pitch;
                self.frame(centre, radius);
            }
        }
    }
}

/// Imperative camera command issued from the parent component.
///
/// Used together with the `camera_command` + `camera_command_seq` props on
/// [`crate::graph3d::Graph3D`] to snap the orbit camera to a specific
/// perspective (e.g. "top-down for a 2-D tree layout") without re-mounting
/// the component.  The `seq` value is a monotonic generation counter; each
/// time the parent wants to (re)apply the command \u2014 even if the command
/// value itself is unchanged \u2014 it must increment the counter so the child
/// can detect it via a `use_hook` "last applied seq" tracker.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CameraCommand {
    /// Restore the default orbit angle and frame the entire layout.
    ResetToDefault,
    /// Snap to the given yaw / pitch (radians) and frame the entire layout.
    ResetTo { yaw: f32, pitch: f32 },
}

#[derive(Debug, Clone, Default)]
pub struct MouseState {
    pub orbiting: bool,
    pub panning:  bool,
    pub last_x:   f64,
    pub last_y:   f64,
}

//! Orbit camera + mouse interaction state.

use std::f32::consts::{
    PI,
    TAU,
};

/// Which layout algorithm the caller is using.  Stored here (in viewer-api)
/// so the built-in settings panel can display and trigger layout changes.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum LayoutMode {
    /// Hierarchical BFS rows on the Y axis, force-directed spread in the XZ
    /// plane — full 3-D depth cues.
    #[default]
    Hierarchical3D,
    /// Same hierarchical rows but Z coordinates zeroed — flat top-down view.
    Flat2D,
}

/// Camera projection mode.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Projection {
    #[default]
    Perspective,
    /// Orthographic projection (no perspective foreshortening).
    Orthographic,
}

/// Interactive camera behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CameraMode {
    /// Orbit around a world-space target.
    #[default]
    Orbit,
    /// Rotate in place and translate through the scene.
    Free,
}

impl CameraMode {
    pub const ALL: [CameraMode; 2] = [CameraMode::Orbit, CameraMode::Free];

    pub fn as_str(self) -> &'static str {
        match self {
            CameraMode::Orbit => "orbit",
            CameraMode::Free => "free",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            CameraMode::Orbit => "Orbit",
            CameraMode::Free => "Free",
        }
    }

    pub fn from_str_opt(value: &str) -> Option<Self> {
        match value {
            "orbit" => Some(CameraMode::Orbit),
            "free" => Some(CameraMode::Free),
            _ => None,
        }
    }
}

/// Vertical FOV (radians) — 45°.
pub const CAMERA_FOV: f32 = std::f32::consts::FRAC_PI_4;
pub const CAMERA_NEAR: f32 = 0.1;
pub const CAMERA_FAR: f32 = 800.0;
pub const CAMERA_MIN_DISTANCE: f32 = 3.0;
pub const CAMERA_MAX_DISTANCE: f32 = 320.0;
pub const CAMERA_MIN_FOCUS_DISTANCE: f32 = 6.0;
pub const CAMERA_PITCH_LIMIT: f32 = 1.4;

const CAMERA_ROTATE_SENSITIVITY: f32 = 0.005;
const CAMERA_SCREEN_PAN_SPEED: f32 = 0.002;

/// viewProj(64) + eye(16) + time(16) = 96 bytes = 24 floats.
pub const CAM_UNIFORM_FLOATS: usize = 24;

/// ThemePalette: 24 × vec4f = 96 floats. (Unused by the edge shader, but
/// the pipeline declares the binding so the buffer must exist.)
pub const PALETTE_FLOATS: usize = 96;

#[derive(Debug, Clone, PartialEq)]
pub struct Camera {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub target: [f32; 3],
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            yaw: 0.3,
            pitch: 0.4,
            distance: 25.0,
            target: [0.0, 0.0, -4.0],
        }
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

    pub fn forward(&self) -> [f32; 3] {
        forward_from_angles(self.yaw, self.pitch)
    }

    pub fn right(&self) -> [f32; 3] {
        let raw_right = cross(self.forward(), [0.0, 1.0, 0.0]);
        if length(raw_right) < 1e-6 {
            [1.0, 0.0, 0.0]
        } else {
            normalise(raw_right)
        }
    }

    pub fn up(&self) -> [f32; 3] {
        normalise(cross(self.right(), self.forward()))
    }

    /// Frame the camera so a sphere of radius `radius` around `centre` is
    /// fully visible.
    pub fn frame(
        &mut self,
        centre: [f32; 3],
        radius: f32,
    ) {
        self.target = centre;
        self.distance = frame_distance(radius);
    }

    /// Retarget the orbit camera and zoom to a closer distance.
    pub fn focus(
        &mut self,
        target: [f32; 3],
        distance: f32,
    ) {
        self.target = target;
        self.distance =
            distance.clamp(CAMERA_MIN_FOCUS_DISTANCE, CAMERA_MAX_DISTANCE);
    }

    pub fn zoom_by_factor(
        &mut self,
        factor: f32,
    ) {
        self.distance =
            (self.distance * factor).clamp(CAMERA_MIN_DISTANCE, CAMERA_MAX_DISTANCE);
    }

    pub fn orbit_by(
        &mut self,
        dx: f32,
        dy: f32,
    ) {
        self.yaw -= dx * CAMERA_ROTATE_SENSITIVITY;
        self.pitch = (self.pitch + dy * CAMERA_ROTATE_SENSITIVITY)
            .clamp(-CAMERA_PITCH_LIMIT, CAMERA_PITCH_LIMIT);
    }

    pub fn set_orientation_in_place(
        &mut self,
        yaw: f32,
        pitch: f32,
    ) {
        let eye = self.eye();
        self.yaw = yaw;
        self.pitch = pitch.clamp(-CAMERA_PITCH_LIMIT, CAMERA_PITCH_LIMIT);
        let forward = self.forward();
        self.target = [
            eye[0] + forward[0] * self.distance,
            eye[1] + forward[1] * self.distance,
            eye[2] + forward[2] * self.distance,
        ];
    }

    pub fn rotate_in_place(
        &mut self,
        dx: f32,
        dy: f32,
    ) {
        let next_yaw = self.yaw - dx * CAMERA_ROTATE_SENSITIVITY;
        let next_pitch = self.pitch + dy * CAMERA_ROTATE_SENSITIVITY;
        self.set_orientation_in_place(next_yaw, next_pitch);
    }

    pub fn translate(
        &mut self,
        delta: [f32; 3],
    ) {
        for (target_axis, delta_axis) in self.target.iter_mut().zip(delta) {
            *target_axis += delta_axis;
        }
    }

    pub fn pan_screen_plane(
        &mut self,
        dx: f32,
        dy: f32,
    ) {
        let speed = self.distance * CAMERA_SCREEN_PAN_SPEED;
        let right = self.right();
        let up = self.up();
        self.translate([
            -right[0] * dx * speed + up[0] * dy * speed,
            -right[1] * dx * speed + up[1] * dy * speed,
            -right[2] * dx * speed + up[2] * dy * speed,
        ]);
    }

    pub fn move_forward(
        &mut self,
        distance: f32,
    ) {
        let forward = self.forward();
        self.translate([
            forward[0] * distance,
            forward[1] * distance,
            forward[2] * distance,
        ]);
    }

    /// Apply a `CameraCommand` to this camera.
    ///
    /// Only the orbit *orientation* (yaw / pitch) is reset; the existing
    /// `distance` and `target` are intentionally preserved so that a
    /// "reset perspective" gesture does not also undo the user's zoom
    /// or pan. Some commands operate on a specific target instead of the
    /// layout bounds, so `_bounds` remains optional context.
    pub fn apply_command(
        &mut self,
        cmd: &CameraCommand,
        _bounds: ([f32; 3], f32),
    ) {
        self.apply_command_for_mode(cmd, CameraMode::Orbit, _bounds);
    }

    pub fn apply_command_for_mode(
        &mut self,
        cmd: &CameraCommand,
        mode: CameraMode,
        _bounds: ([f32; 3], f32),
    ) {
        match *cmd {
            CameraCommand::ResetToDefault => {
                let def = Camera::default();
                if mode == CameraMode::Free {
                    self.set_orientation_in_place(def.yaw, def.pitch);
                } else {
                    self.yaw = def.yaw;
                    self.pitch = def.pitch;
                }
            },
            CameraCommand::ResetTo { yaw, pitch } => {
                if mode == CameraMode::Free {
                    self.set_orientation_in_place(yaw, pitch);
                } else {
                    self.yaw = yaw;
                    self.pitch = pitch;
                }
            },
            CameraCommand::FocusOn { target, distance } =>
                self.focus(target, distance),
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
    /// Retarget the existing orbit to a world-space point and zoom in.
    FocusOn { target: [f32; 3], distance: f32 },
}

pub fn frame_distance(radius: f32) -> f32 {
    let half_fov_tan = (CAMERA_FOV * 0.5).tan();
    ((radius / half_fov_tan) * 1.3).clamp(12.0, CAMERA_MAX_DISTANCE)
}

pub fn animate_camera(
    camera: &mut Camera,
    goal: &Camera,
    dt: f32,
    lerp_speed: f32,
) -> bool {
    let alpha = 1.0 - (-lerp_speed * dt.max(0.0)).exp();
    let epsilon = 0.0001;

    let yaw_delta = shortest_angle_delta(camera.yaw, goal.yaw);
    let pitch_delta = goal.pitch - camera.pitch;
    let distance_delta = goal.distance - camera.distance;
    let target_delta = [
        goal.target[0] - camera.target[0],
        goal.target[1] - camera.target[1],
        goal.target[2] - camera.target[2],
    ];

    camera.yaw += yaw_delta * alpha;
    camera.pitch += pitch_delta * alpha;
    camera.distance += distance_delta * alpha;
    camera.target[0] += target_delta[0] * alpha;
    camera.target[1] += target_delta[1] * alpha;
    camera.target[2] += target_delta[2] * alpha;

    let remaining_yaw = shortest_angle_delta(camera.yaw, goal.yaw);
    let remaining_pitch = goal.pitch - camera.pitch;
    let remaining_distance = goal.distance - camera.distance;
    let remaining_target = [
        goal.target[0] - camera.target[0],
        goal.target[1] - camera.target[1],
        goal.target[2] - camera.target[2],
    ];
    let done = remaining_yaw.abs() < epsilon
        && remaining_pitch.abs() < epsilon
        && remaining_distance.abs() < epsilon
        && remaining_target.iter().all(|delta| delta.abs() < epsilon);
    if done {
        *camera = goal.clone();
    }

    !done
}

fn shortest_angle_delta(
    current: f32,
    target: f32,
) -> f32 {
    (target - current + PI).rem_euclid(TAU) - PI
}

fn forward_from_angles(
    yaw: f32,
    pitch: f32,
) -> [f32; 3] {
    let cp = pitch.cos();
    normalise([-cp * yaw.sin(), -pitch.sin(), -cp * yaw.cos()])
}

fn cross(
    a: [f32; 3],
    b: [f32; 3],
) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn length(vector: [f32; 3]) -> f32 {
    (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2])
        .sqrt()
}

fn normalise(vector: [f32; 3]) -> [f32; 3] {
    let length = length(vector);
    if length < 1e-6 {
        [0.0, 0.0, -1.0]
    } else {
        [vector[0] / length, vector[1] / length, vector[2] / length]
    }
}

#[derive(Debug, Clone, Default)]
pub struct MouseState {
    pub orbiting: bool,
    pub panning: bool,
    pub last_x: f64,
    pub last_y: f64,
}

#[cfg(test)]
mod tests {
    use super::{
        animate_camera,
        Camera,
        CameraMode,
        CameraCommand,
        frame_distance,
    };

    fn assert_vec3_close(
        actual: [f32; 3],
        expected: [f32; 3],
    ) {
        for axis in 0..3 {
            assert!((actual[axis] - expected[axis]).abs() < 1e-4);
        }
    }

    #[test]
    fn focus_command_recenters_without_resetting_orbit() {
        let mut camera = Camera {
            yaw: 1.2,
            pitch: 0.6,
            distance: 28.0,
            target: [0.0, 0.0, 0.0],
        };

        camera.apply_command(
            &CameraCommand::FocusOn {
                target: [4.0, -2.0, 7.5],
                distance: 8.5,
            },
            ([0.0, 0.0, 0.0], 1.0),
        );

        assert_eq!(camera.target, [4.0, -2.0, 7.5]);
        assert!((camera.distance - 8.5).abs() < f32::EPSILON);
        assert!((camera.yaw - 1.2).abs() < f32::EPSILON);
        assert!((camera.pitch - 0.6).abs() < f32::EPSILON);
    }

    #[test]
    fn focus_command_clamps_to_minimum_distance() {
        let mut camera = Camera::default();

        camera.apply_command(
            &CameraCommand::FocusOn {
                target: [1.0, 2.0, 3.0],
                distance: 1.0,
            },
            ([0.0, 0.0, 0.0], 1.0),
        );

        assert_eq!(camera.target, [1.0, 2.0, 3.0]);
        assert!((camera.distance - 6.0).abs() < f32::EPSILON);
    }

    #[test]
    fn animate_camera_moves_toward_goal_and_snaps_when_close() {
        let mut camera = Camera {
            yaw: 0.3,
            pitch: 0.2,
            distance: 20.0,
            target: [0.0, 0.0, 0.0],
        };
        let goal = Camera {
            yaw: 0.8,
            pitch: 0.5,
            distance: 8.5,
            target: [4.0, -1.0, 7.0],
        };

        let moved = animate_camera(&mut camera, &goal, 1.0 / 60.0, 6.0);
        assert!(moved);
        assert!(camera.distance < 20.0);
        assert!(camera.target[0] > 0.0);

        for _ in 0..180 {
            if !animate_camera(&mut camera, &goal, 1.0 / 60.0, 6.0) {
                break;
            }
        }

        assert_eq!(camera.target, goal.target);
        assert!((camera.distance - goal.distance).abs() < f32::EPSILON);
        assert!((camera.yaw - goal.yaw).abs() < f32::EPSILON);
        assert!((camera.pitch - goal.pitch).abs() < f32::EPSILON);
    }

    #[test]
    fn free_rotation_preserves_eye_position() {
        let mut camera = Camera {
            yaw: 0.8,
            pitch: -0.35,
            distance: 32.0,
            target: [4.0, -1.5, 7.0],
        };
        let eye_before = camera.eye();

        camera.rotate_in_place(84.0, -38.0);

        let eye_after = camera.eye();
        assert_vec3_close(eye_after, eye_before);
    }

    #[test]
    fn free_move_forward_translates_eye_and_target_together() {
        let mut camera = Camera::default();
        let eye_before = camera.eye();
        let target_before = camera.target;
        let forward = camera.forward();

        camera.move_forward(7.5);

        let eye_after = camera.eye();
        let target_after = camera.target;
        assert_vec3_close(
            eye_after,
            [
                eye_before[0] + forward[0] * 7.5,
                eye_before[1] + forward[1] * 7.5,
                eye_before[2] + forward[2] * 7.5,
            ],
        );
        assert_vec3_close(
            target_after,
            [
                target_before[0] + forward[0] * 7.5,
                target_before[1] + forward[1] * 7.5,
                target_before[2] + forward[2] * 7.5,
            ],
        );
    }

    #[test]
    fn free_reset_camera_reorients_in_place() {
        let mut camera = Camera {
            yaw: 1.1,
            pitch: 0.2,
            distance: 18.0,
            target: [3.0, 5.0, -4.0],
        };
        let eye_before = camera.eye();

        camera.apply_command_for_mode(
            &CameraCommand::ResetToDefault,
            CameraMode::Free,
            ([0.0, 0.0, 0.0], 1.0),
        );

        assert_vec3_close(camera.eye(), eye_before);
    }

    #[test]
    fn frame_distance_supports_further_zoom_out() {
        assert!(frame_distance(200.0) > 120.0);
    }
}

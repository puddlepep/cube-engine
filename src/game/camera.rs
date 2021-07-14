use std::f32::consts::PI;

use cgmath::Rad;
use cgmath::Vector3;
use cgmath::Matrix4;

// The coordinate system in wgpu is based on DirectX and Metal's coordinate systems, which are in
// normalized device coordinates. The x and y axis being in the range of -1.0 to 1.0, and the z
// axis in range of 0.0 to 1.0. The cgmath crate is built for OpenGL's coordinate system, so we must translate.
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub const PITCH_MAX: f32 = PI / 2.0 - 0.01;
pub const PITCH_MIN: f32 = -(PI / 2.0) + 0.01;

pub struct Camera {

    pub position: Vector3<f32>,
    pub yaw: f32,
    pub pitch: f32,

    pub speed: f32,
    pub sensitivity: f32,

    pub fovy: cgmath::Rad<f32>,
    pub near: f32,
    pub far: f32,

    pub width: u32,
    pub height: u32,
}

impl Camera {

    pub fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        
        // Rotating the camera along yaw and pitch.
        // Probably a simpler way to do this but god dammit dude fuck matrices and trigonometry and shit.
        // I'm proud enough that this works lmfao.

        let y_rot = cgmath::Matrix3::from_angle_y(Rad(-self.yaw));
        let dir = y_rot * -Vector3::unit_z();
        let right = dir.cross(Vector3::unit_y());
        let pitch_rot = cgmath::Matrix3::from_axis_angle(right, Rad(self.pitch));
        let dir = pitch_rot * dir;
        
        let view = Matrix4::look_to_rh(cgmath::Point3::new(self.position.x, self.position.y, self.position.z), dir, Vector3::unit_y());
        let projection = cgmath::perspective(self.fovy, self.width as f32 / self.height as f32, self.near, self.far);

        return OPENGL_TO_WGPU_MATRIX * projection * view;

    }

    pub fn build_ui_projection_matrix(&self) -> Matrix4<f32> {

        let projection = cgmath::ortho(0.0, self.width as f32, self.height as f32, 0.0, -1.0, 1.0);
        return projection;

    }

    // Returns all (important) normalized directions of the camera.
    // -> ( forward, right, up )
    pub fn get_headings(&mut self) -> ( Vector3<f32>, Vector3<f32>, Vector3<f32> ) {
        
        let y_rot = cgmath::Matrix3::from_angle_y(Rad(-self.yaw));
        let forward = y_rot * -Vector3::unit_z();
        let right = forward.cross(Vector3::unit_y());
        let p_rot = cgmath::Matrix3::from_axis_angle(right, Rad(self.pitch));
        let forward = p_rot * forward;
        let up = -forward.cross(right);

        (forward, right, up)

    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

}

use cgmath::{InnerSpace, Vector3};
use super::Camera;

pub struct Plane {
    pub point: Vector3<f32>,
    pub normal: Vector3<f32>,
}

impl Plane {
    pub fn new() -> Plane {
        Plane {
            point: Vector3::new(0.0, 0.0, 0.0),
            normal: Vector3::new(1.0, 0.0, 0.0),
        }
    }
}

pub struct Frustum {
    pub planes: [Plane; 6],
}

impl Frustum {
 
    pub fn new() -> Frustum {
        Frustum {
            planes: [
                Plane::new(),
                Plane::new(),
                Plane::new(),
                Plane::new(),
                Plane::new(),
                Plane::new(),
            ]
        }
    }

    pub fn get_planes(&self, camera: &Camera) -> [Plane; 6] {
        
        let (forward, right, up) = camera.get_headings();
        let ratio = camera.width as f32 / camera.height as f32;

        let near_height = 2.0 * f32::tan(camera.fovy.0 / 2.0) * camera.near;
        let near_width = near_height * ratio;

        //let far_height = 2.0 * f32::tan(camera.fovy.0 / 2.0) * camera.far;
        //let far_width = far_height * ratio;

        // the centers of the far and near planes.
        let fc = camera.position + forward * camera.far;
        let nc = camera.position + forward * camera.near;

        // generating directions from the center of the camera to the edges of the near plane
        let right_dir = ((nc + right * near_width / 2.0) - camera.position).normalize();
        let left_dir = ((nc - right * near_width / 2.0) - camera.position).normalize();
        let top_dir = ((nc + up * near_height / 2.0) - camera.position).normalize();
        let bottom_dir = ((nc - up * near_height / 2.0) - camera.position).normalize();

        // generating normals from the previous info
        let right_normal = up.cross(right_dir);
        let left_normal = -up.cross(left_dir);
        let top_normal = -right.cross(top_dir);
        let bottom_normal = right.cross(bottom_dir);

        // updating planes
        let near_plane = Plane { point: nc, normal: forward };
        let far_plane = Plane { point: fc, normal: -forward };

        let right_plane = Plane { point: nc + right * near_width / 2.0, normal: right_normal };
        let left_plane = Plane { point: nc - right * near_width / 2.0, normal: left_normal };

        let top_plane = Plane { point: nc + up * near_height / 2.0, normal: top_normal };
        let bottom_plane = Plane { point: nc - up * near_height / 2.0, normal: bottom_normal };

        [near_plane, far_plane, right_plane, left_plane, top_plane, bottom_plane]
    }

    pub fn sphere_intersection(&self, point: Vector3<f32>, radius: f32) -> bool {


        
        for plane in &self.planes {

            let dist = plane.normal.dot(point - plane.point);

            if dist < -radius {
                return false;
            }
        }

        true

    }

}
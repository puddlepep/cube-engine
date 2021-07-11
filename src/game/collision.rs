
use cgmath::{MetricSpace, Vector3};

pub struct CollisionInfo {

}

pub struct CircleCollider {
    pub position: Vector3<f32>,
    pub radius: f32,
}

impl CircleCollider {
    pub fn new(position: Vector3<f32>, radius: f32) -> CircleCollider {
        CircleCollider {
            position: position,
            radius
        }
    }
}

pub struct CubeCollider {
    pub position: Vector3<f32>,
    pub size: Vector3<f32>,
}

impl CubeCollider {
    pub fn new(position: Vector3<f32>, size: Vector3<f32>) -> CubeCollider {
        CubeCollider {
            position,
            size,
        }
    }
}

#[allow(dead_code)]
pub fn circle_circle(c0: &CircleCollider, c1: &CircleCollider) -> Option<CollisionInfo> {
    
    // Just check the sum of the two circle's radii and make sure they're on the same y-level.
    let distance = c0.position.distance(c1.position);
    if distance <= c0.radius + c1.radius && c0.position.y == c1.position.y {
        Some(CollisionInfo { })
    }
    else {
        None
    }

}

pub fn circle_cube(circle: &CircleCollider, cube: &CubeCollider) -> Option<CollisionInfo> {

    // If the circle's y-value is within the cube's y extents.
    if circle.position.y >= cube.position.y && circle.position.y < cube.position.y + cube.size.y {

        let mut test_pos = circle.position;

        if circle.position.x < cube.position.x {
            test_pos.x = cube.position.x;
        }
        else if circle.position.x > cube.position.x + cube.size.x {
            test_pos.x = cube.position.x + cube.size.x;
        }

        if circle.position.z > cube.position.z {
            test_pos.z = cube.position.z;
        }
        else if circle.position.z < cube.position.z + cube.size.z {
            test_pos.z = cube.position.z + cube.size.z;
        }

        let distance = circle.position.distance(test_pos);
        if distance <= circle.radius {
            Some(CollisionInfo {})
        }
        else {
            None
        }


    }
    else {
        None
    }

}


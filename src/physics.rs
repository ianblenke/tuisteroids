// Physics capability: Vec2, motion integration, toroidal wrapping, angular rotation

use std::f64::consts::PI;
use std::ops::{Add, Sub};

/// A 2D vector with f64 components.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn scale(self, scalar: f64) -> Vec2 {
        Vec2::new(self.x * scalar, self.y * scalar)
    }

    pub fn magnitude(self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(self) -> Vec2 {
        let mag = self.magnitude();
        if mag == 0.0 {
            Vec2::new(0.0, 0.0)
        } else {
            self.scale(1.0 / mag)
        }
    }

    pub fn dot(self, other: Vec2) -> f64 {
        self.x * other.x + self.y * other.y
    }

    pub fn from_angle(angle: f64) -> Vec2 {
        Vec2::new(angle.cos(), angle.sin())
    }
}

impl Add for Vec2 {
    type Output = Vec2;

    fn add(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for Vec2 {
    type Output = Vec2;

    fn sub(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x - other.x, self.y - other.y)
    }
}

/// Update position using Euler integration: position = position + velocity * dt
pub fn integrate_motion(position: Vec2, velocity: Vec2, dt: f64) -> Vec2 {
    position + velocity.scale(dt)
}

/// Apply drag to velocity: velocity = velocity * drag_factor
pub fn apply_drag(velocity: Vec2, drag_factor: f64) -> Vec2 {
    velocity.scale(drag_factor)
}

/// Wrap a position into the world bounds [0, width) x [0, height)
pub fn wrap_position(position: Vec2, width: f64, height: f64) -> Vec2 {
    let x = ((position.x % width) + width) % width;
    let y = ((position.y % height) + height) % height;
    Vec2::new(x, y)
}

/// Rotate an angle by angular velocity over dt, normalized to [0, 2*PI)
pub fn rotate_angle(angle: f64, angular_velocity: f64, dt: f64) -> f64 {
    let new_angle = angle + angular_velocity * dt;
    let two_pi = 2.0 * PI;
    ((new_angle % two_pi) + two_pi) % two_pi
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    const EPSILON: f64 = 1e-10;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < EPSILON
    }

    fn vec2_approx_eq(a: Vec2, b: Vec2) -> bool {
        approx_eq(a.x, b.x) && approx_eq(a.y, b.y)
    }

    // === Requirement: Two-Dimensional Vector Type ===

    // Scenario: Vector addition
    #[test]
    fn test_vector_addition() {
        let a = Vec2::new(3.0, 4.0);
        let b = Vec2::new(1.0, 2.0);
        let result = a + b;
        assert_eq!(result, Vec2::new(4.0, 6.0));
    }

    // Scenario: Vector subtraction
    #[test]
    fn test_vector_subtraction() {
        let a = Vec2::new(3.0, 4.0);
        let b = Vec2::new(1.0, 2.0);
        let result = a - b;
        assert_eq!(result, Vec2::new(2.0, 2.0));
    }

    // Scenario: Scalar multiplication
    #[test]
    fn test_scalar_multiplication() {
        let a = Vec2::new(3.0, 4.0);
        let result = a.scale(2.0);
        assert_eq!(result, Vec2::new(6.0, 8.0));
    }

    // Scenario: Vector magnitude
    #[test]
    fn test_vector_magnitude() {
        let a = Vec2::new(3.0, 4.0);
        assert_eq!(a.magnitude(), 5.0);
    }

    // Scenario: Vector normalization
    #[test]
    fn test_vector_normalization() {
        let a = Vec2::new(3.0, 4.0);
        let result = a.normalize();
        assert!(vec2_approx_eq(result, Vec2::new(0.6, 0.8)));
        assert!(approx_eq(result.magnitude(), 1.0));
    }

    // Scenario: Zero vector normalization
    #[test]
    fn test_zero_vector_normalization() {
        let a = Vec2::new(0.0, 0.0);
        let result = a.normalize();
        assert_eq!(result, Vec2::new(0.0, 0.0));
    }

    // Scenario: Dot product
    #[test]
    fn test_dot_product() {
        let a = Vec2::new(1.0, 0.0);
        let b = Vec2::new(0.0, 1.0);
        assert_eq!(a.dot(b), 0.0);
    }

    // Scenario: Vector from angle zero
    #[test]
    fn test_vector_from_angle_zero() {
        let result = Vec2::from_angle(0.0);
        assert!(vec2_approx_eq(result, Vec2::new(1.0, 0.0)));
    }

    // Scenario: Vector from angle 90 degrees
    #[test]
    fn test_vector_from_angle_90() {
        let result = Vec2::from_angle(PI / 2.0);
        assert!(vec2_approx_eq(result, Vec2::new(0.0, 1.0)));
    }

    // === Requirement: Motion Integration ===

    // Scenario: Position update with constant velocity
    #[test]
    fn test_position_update_constant_velocity() {
        let pos = Vec2::new(100.0, 100.0);
        let vel = Vec2::new(60.0, 0.0);
        let dt = 1.0 / 60.0;
        let result = integrate_motion(pos, vel, dt);
        assert!(vec2_approx_eq(result, Vec2::new(101.0, 100.0)));
    }

    // Scenario: Position update with zero velocity
    #[test]
    fn test_position_update_zero_velocity() {
        let pos = Vec2::new(50.0, 50.0);
        let vel = Vec2::new(0.0, 0.0);
        let dt = 1.0 / 60.0;
        let result = integrate_motion(pos, vel, dt);
        assert_eq!(result, Vec2::new(50.0, 50.0));
    }

    // === Requirement: Velocity Dampening ===

    // Scenario: Velocity decreases over time with drag
    #[test]
    fn test_velocity_drag() {
        let vel = Vec2::new(100.0, 0.0);
        let result = apply_drag(vel, 0.99);
        assert!(approx_eq(result.magnitude(), 99.0));
    }

    // Scenario: Stationary entity stays stationary
    #[test]
    fn test_stationary_drag() {
        let vel = Vec2::new(0.0, 0.0);
        let result = apply_drag(vel, 0.99);
        assert_eq!(result, Vec2::new(0.0, 0.0));
    }

    // === Requirement: Toroidal Wrapping ===

    // Scenario: Wrap right edge to left
    #[test]
    fn test_wrap_right_to_left() {
        let pos = Vec2::new(801.0, 300.0);
        let result = wrap_position(pos, 800.0, 600.0);
        assert!(approx_eq(result.x, 1.0));
        assert!(approx_eq(result.y, 300.0));
    }

    // Scenario: Wrap left edge to right
    #[test]
    fn test_wrap_left_to_right() {
        let pos = Vec2::new(-1.0, 300.0);
        let result = wrap_position(pos, 800.0, 600.0);
        assert!(approx_eq(result.x, 799.0));
        assert!(approx_eq(result.y, 300.0));
    }

    // Scenario: Wrap bottom edge to top
    #[test]
    fn test_wrap_bottom_to_top() {
        let pos = Vec2::new(400.0, 601.0);
        let result = wrap_position(pos, 800.0, 600.0);
        assert!(approx_eq(result.x, 400.0));
        assert!(approx_eq(result.y, 1.0));
    }

    // Scenario: Wrap top edge to bottom
    #[test]
    fn test_wrap_top_to_bottom() {
        let pos = Vec2::new(400.0, -1.0);
        let result = wrap_position(pos, 800.0, 600.0);
        assert!(approx_eq(result.x, 400.0));
        assert!(approx_eq(result.y, 599.0));
    }

    // Scenario: No wrapping when inside bounds
    #[test]
    fn test_no_wrapping_inside_bounds() {
        let pos = Vec2::new(400.0, 300.0);
        let result = wrap_position(pos, 800.0, 600.0);
        assert!(approx_eq(result.x, 400.0));
        assert!(approx_eq(result.y, 300.0));
    }

    // === Requirement: Angular Rotation ===

    // Scenario: Rotate clockwise
    #[test]
    fn test_rotate_clockwise() {
        let angle = rotate_angle(0.0, PI, 1.0);
        assert!(approx_eq(angle, PI));
    }

    // Scenario: Angle wraps past 2*PI
    #[test]
    fn test_angle_wraps_past_two_pi() {
        let angle = rotate_angle(6.0, 1.0, 1.0);
        let expected = 7.0 - 2.0 * PI;
        assert!(approx_eq(angle, expected));
    }
}

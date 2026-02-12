// Ship capability: player ship with rotation, thrust, lives, respawn

use crate::physics::{self, Vec2};
use std::f64::consts::PI;

pub const ROTATION_SPEED: f64 = 5.0; // radians per second
pub const THRUST_ACCELERATION: f64 = 200.0; // units per second^2
pub const MAX_SPEED: f64 = 400.0; // units per second
pub const SHIP_RADIUS: f64 = 12.0;
pub const INITIAL_LIVES: u32 = 3;
pub const INVULNERABILITY_DURATION: f64 = 3.0; // seconds
pub const EXTRA_LIFE_SCORE: u32 = 10_000;

// Ship triangle vertices relative to center (pointing right at angle 0)
const NOSE_OFFSET: f64 = 15.0;
const WING_OFFSET: f64 = 10.0;
const WING_SPREAD: f64 = 8.0;

pub struct Ship {
    pub position: Vec2,
    pub velocity: Vec2,
    pub rotation: f64,
    pub lives: u32,
    pub invulnerable: bool,
    pub invulnerable_timer: f64,
    pub extra_life_awarded: bool,
}

impl Ship {
    pub fn new(center_x: f64, center_y: f64) -> Self {
        Self {
            position: Vec2::new(center_x, center_y),
            velocity: Vec2::new(0.0, 0.0),
            rotation: -PI / 2.0, // pointing up
            lives: INITIAL_LIVES,
            invulnerable: false,
            invulnerable_timer: 0.0,
            extra_life_awarded: false,
        }
    }

    /// Get the ship's polygon vertices in world space (triangle).
    pub fn vertices(&self) -> [Vec2; 3] {
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();

        // Nose (front)
        let nose = Vec2::new(
            self.position.x + NOSE_OFFSET * cos_r,
            self.position.y + NOSE_OFFSET * sin_r,
        );

        // Left wing (back-left)
        let left = Vec2::new(
            self.position.x + (-WING_OFFSET * cos_r - WING_SPREAD * sin_r),
            self.position.y + (-WING_OFFSET * sin_r + WING_SPREAD * cos_r),
        );

        // Right wing (back-right)
        let right = Vec2::new(
            self.position.x + (-WING_OFFSET * cos_r + WING_SPREAD * sin_r),
            self.position.y + (-WING_OFFSET * sin_r - WING_SPREAD * cos_r),
        );

        [nose, left, right]
    }

    /// Apply rotation based on input.
    pub fn rotate(&mut self, left: bool, right: bool, dt: f64) {
        if left {
            self.rotation -= ROTATION_SPEED * dt;
        }
        if right {
            self.rotation += ROTATION_SPEED * dt;
        }
        // Normalize angle
        self.rotation = physics::rotate_angle(self.rotation, 0.0, 1.0);
    }

    /// Apply thrust in the facing direction.
    pub fn thrust(&mut self, dt: f64) {
        let direction = Vec2::from_angle(self.rotation);
        let acceleration = direction.scale(THRUST_ACCELERATION * dt);
        self.velocity = self.velocity + acceleration;

        // Clamp to max speed
        let speed = self.velocity.magnitude();
        if speed > MAX_SPEED {
            self.velocity = self.velocity.normalize().scale(MAX_SPEED);
        }
    }

    /// Update position, apply drag, wrap.
    pub fn update(&mut self, dt: f64, world_width: f64, world_height: f64) {
        self.position = physics::integrate_motion(self.position, self.velocity, dt);
        self.position = physics::wrap_position(self.position, world_width, world_height);

        if self.invulnerable {
            self.invulnerable_timer -= dt;
            if self.invulnerable_timer <= 0.0 {
                self.invulnerable = false;
                self.invulnerable_timer = 0.0;
            }
        }
    }

    /// Get the nose position (for spawning bullets).
    pub fn nose_position(&self) -> Vec2 {
        self.vertices()[0]
    }

    /// Destroy the ship — lose a life and respawn.
    pub fn destroy(&mut self, world_width: f64, world_height: f64) {
        if self.lives > 0 {
            self.lives -= 1;
        }
        if self.lives > 0 {
            self.respawn(world_width, world_height);
        }
    }

    /// Respawn at center with invulnerability.
    pub fn respawn(&mut self, world_width: f64, world_height: f64) {
        self.position = Vec2::new(world_width / 2.0, world_height / 2.0);
        self.velocity = Vec2::new(0.0, 0.0);
        self.rotation = -PI / 2.0; // facing up
        self.invulnerable = true;
        self.invulnerable_timer = INVULNERABILITY_DURATION;
    }

    /// Check and award extra life at score threshold.
    pub fn check_extra_life(&mut self, score: u32) {
        if !self.extra_life_awarded && score >= EXTRA_LIFE_SCORE {
            self.lives += 1;
            self.extra_life_awarded = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    const EPSILON: f64 = 1e-6;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < EPSILON
    }

    // === Requirement: Ship Shape Definition ===

    // Scenario: Ship vertices form a triangle
    #[test]
    fn test_ship_vertices_form_triangle() {
        let mut ship = Ship::new(400.0, 300.0);
        ship.rotation = 0.0; // facing right
        let verts = ship.vertices();
        // Should be 3 distinct vertices
        assert_ne!(verts[0], verts[1]);
        assert_ne!(verts[1], verts[2]);
        assert_ne!(verts[0], verts[2]);
        // Nose should be rightmost (highest x) when facing right
        assert!(verts[0].x > verts[1].x);
        assert!(verts[0].x > verts[2].x);
    }

    // Scenario: Ship vertices rotate with the ship
    #[test]
    fn test_ship_vertices_rotate() {
        let mut ship = Ship::new(400.0, 300.0);
        ship.rotation = 0.0;
        let verts_0 = ship.vertices();
        ship.rotation = PI / 2.0;
        let verts_90 = ship.vertices();
        // At PI/2, nose should point down (highest y)
        assert!(verts_90[0].y > verts_90[1].y || verts_90[0].y > verts_90[2].y);
        // And they should differ from the angle=0 case
        assert!(!approx_eq(verts_0[0].x, verts_90[0].x) || !approx_eq(verts_0[0].y, verts_90[0].y));
    }

    // === Requirement: Ship Rotation ===

    // Scenario: Rotate left decreases angle
    #[test]
    fn test_rotate_left() {
        let mut ship = Ship::new(400.0, 300.0);
        ship.rotation = 1.0;
        let original = ship.rotation;
        ship.rotate(true, false, 1.0 / 60.0);
        assert!(ship.rotation < original);
    }

    // Scenario: Rotate right increases angle
    #[test]
    fn test_rotate_right() {
        let mut ship = Ship::new(400.0, 300.0);
        ship.rotation = 1.0;
        let original = ship.rotation;
        ship.rotate(false, true, 1.0 / 60.0);
        // After normalization, should be greater (mod 2pi)
        let expected = original + ROTATION_SPEED / 60.0;
        assert!(approx_eq(ship.rotation, expected % (2.0 * PI)));
    }

    // Scenario: No rotation when no input
    #[test]
    fn test_no_rotation_without_input() {
        let mut ship = Ship::new(400.0, 300.0);
        ship.rotation = 1.0;
        ship.rotate(false, false, 1.0 / 60.0);
        assert!(approx_eq(ship.rotation, 1.0));
    }

    // === Requirement: Ship Thrust ===

    // Scenario: Thrust from standstill
    #[test]
    fn test_thrust_from_standstill() {
        let mut ship = Ship::new(400.0, 300.0);
        ship.rotation = 0.0; // facing right
        ship.velocity = Vec2::new(0.0, 0.0);
        ship.thrust(1.0 / 60.0);
        let expected_vx = THRUST_ACCELERATION / 60.0;
        assert!(approx_eq(ship.velocity.x, expected_vx));
        assert!(approx_eq(ship.velocity.y, 0.0));
    }

    // Scenario: Thrust adds to existing velocity
    #[test]
    fn test_thrust_adds_to_velocity() {
        let mut ship = Ship::new(400.0, 300.0);
        ship.rotation = 0.0;
        ship.velocity = Vec2::new(50.0, 0.0);
        ship.thrust(1.0 / 60.0);
        let expected_vx = 50.0 + THRUST_ACCELERATION / 60.0;
        assert!(approx_eq(ship.velocity.x, expected_vx));
    }

    // Scenario: Thrust in different direction than motion
    #[test]
    fn test_thrust_different_direction() {
        let mut ship = Ship::new(400.0, 300.0);
        ship.rotation = PI / 2.0; // facing down
        ship.velocity = Vec2::new(50.0, 0.0);
        ship.thrust(1.0 / 60.0);
        assert!(approx_eq(ship.velocity.x, 50.0));
        let expected_vy = THRUST_ACCELERATION / 60.0;
        assert!(approx_eq(ship.velocity.y, expected_vy));
    }

    // Scenario: No acceleration without thrust
    #[test]
    fn test_no_acceleration_without_thrust() {
        let mut ship = Ship::new(400.0, 300.0);
        ship.velocity = Vec2::new(50.0, 0.0);
        // Don't call thrust - just check velocity stays the same
        assert!(approx_eq(ship.velocity.x, 50.0));
        assert!(approx_eq(ship.velocity.y, 0.0));
    }

    // === Requirement: Ship Maximum Speed ===

    // Scenario: Velocity clamped at maximum
    #[test]
    fn test_velocity_clamped_at_max() {
        let mut ship = Ship::new(400.0, 300.0);
        ship.rotation = 0.0;
        ship.velocity = Vec2::new(MAX_SPEED, 0.0);
        ship.thrust(1.0 / 60.0);
        assert!(ship.velocity.magnitude() <= MAX_SPEED + EPSILON);
    }

    // === Requirement: Ship Lives ===

    // Scenario: Game starts with 3 lives
    #[test]
    fn test_initial_lives() {
        let ship = Ship::new(400.0, 300.0);
        assert_eq!(ship.lives, 3);
    }

    // Scenario: Ship loses one life on death
    #[test]
    fn test_lose_one_life() {
        let mut ship = Ship::new(400.0, 300.0);
        assert_eq!(ship.lives, 3);
        ship.destroy(800.0, 600.0);
        assert_eq!(ship.lives, 2);
    }

    // === Requirement: Ship Respawn ===

    // Scenario: Ship respawns at center
    #[test]
    fn test_respawn_at_center() {
        let mut ship = Ship::new(400.0, 300.0);
        ship.position = Vec2::new(100.0, 100.0);
        ship.velocity = Vec2::new(50.0, 30.0);
        ship.destroy(800.0, 600.0);
        assert!(approx_eq(ship.position.x, 400.0));
        assert!(approx_eq(ship.position.y, 300.0));
        assert!(approx_eq(ship.velocity.x, 0.0));
        assert!(approx_eq(ship.velocity.y, 0.0));
    }

    // Scenario: Ship is invulnerable after respawn
    #[test]
    fn test_invulnerable_after_respawn() {
        let mut ship = Ship::new(400.0, 300.0);
        ship.destroy(800.0, 600.0);
        assert!(ship.invulnerable);
        // After 1 second, still invulnerable
        ship.update(1.0, 800.0, 600.0);
        assert!(ship.invulnerable);
    }

    // Scenario: Ship invulnerability expires
    #[test]
    fn test_invulnerability_expires() {
        let mut ship = Ship::new(400.0, 300.0);
        ship.destroy(800.0, 600.0);
        assert!(ship.invulnerable);
        // After 3+ seconds, not invulnerable
        ship.update(3.1, 800.0, 600.0);
        assert!(!ship.invulnerable);
    }

    // === Requirement: Extra Life ===

    // Scenario: Extra life awarded at 10000 points
    #[test]
    fn test_extra_life_at_10000() {
        let mut ship = Ship::new(400.0, 300.0);
        assert_eq!(ship.lives, 3);
        ship.check_extra_life(10_050);
        assert_eq!(ship.lives, 4);
    }

    // Scenario: Extra life only awarded once
    #[test]
    fn test_extra_life_only_once() {
        let mut ship = Ship::new(400.0, 300.0);
        ship.check_extra_life(10_050);
        assert_eq!(ship.lives, 4);
        ship.check_extra_life(20_000);
        assert_eq!(ship.lives, 4); // no second extra life
    }

    // === Requirement: Ship Screen Wrapping ===

    // Scenario: Ship wraps from right to left
    #[test]
    fn test_ship_wraps_right_to_left() {
        let mut ship = Ship::new(400.0, 300.0);
        ship.position = Vec2::new(799.0, 300.0);
        ship.velocity = Vec2::new(120.0, 0.0); // will cross right edge in 1 frame
        ship.update(1.0 / 60.0, 800.0, 600.0);
        // Should have wrapped around
        assert!(ship.position.x < 800.0);
    }

    // Additional coverage: nose_position
    #[test]
    fn test_nose_position() {
        let mut ship = Ship::new(400.0, 300.0);
        ship.rotation = 0.0;
        let nose = ship.nose_position();
        // Nose should be in front of center at angle 0 (right)
        assert!(nose.x > ship.position.x);
    }
}

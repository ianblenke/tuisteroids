// Bullets capability: projectile creation, lifetime, speed, screen limit

use crate::physics::{self, Vec2};

pub const BULLET_SPEED: f64 = 500.0; // units per second
pub const BULLET_RANGE_FRACTION: f64 = 0.8; // bullets travel 80% of world width (matches original Asteroids)
pub const MAX_BULLETS: usize = 4;
pub const BULLET_RADIUS: f64 = 2.0;

pub struct Bullet {
    pub position: Vec2,
    pub velocity: Vec2,
    pub distance_traveled: f64,
    pub alive: bool,
}

impl Bullet {
    /// Create a bullet at the given position traveling in the given direction.
    pub fn new(position: Vec2, angle: f64) -> Self {
        let velocity = Vec2::from_angle(angle).scale(BULLET_SPEED);
        Self {
            position,
            velocity,
            distance_traveled: 0.0,
            alive: true,
        }
    }

    /// Update bullet position and lifetime (distance-based, matching original Asteroids).
    pub fn update(&mut self, dt: f64, world_width: f64, world_height: f64) {
        self.position = physics::integrate_motion(self.position, self.velocity, dt);
        self.position = physics::wrap_position(self.position, world_width, world_height);
        self.distance_traveled += self.velocity.magnitude() * dt;
        if self.distance_traveled >= world_width * BULLET_RANGE_FRACTION {
            self.alive = false;
        }
    }
}

/// Manages the collection of active bullets.
pub struct BulletPool {
    pub bullets: Vec<Bullet>,
}

impl Default for BulletPool {
    fn default() -> Self {
        Self::new()
    }
}

impl BulletPool {
    pub fn new() -> Self {
        Self {
            bullets: Vec::new(),
        }
    }

    /// Try to fire a new bullet. Returns false if at max capacity.
    pub fn fire(&mut self, position: Vec2, angle: f64) -> bool {
        if self.active_count() >= MAX_BULLETS {
            return false;
        }
        self.bullets.push(Bullet::new(position, angle));
        true
    }

    /// Count of active (alive) bullets.
    pub fn active_count(&self) -> usize {
        self.bullets.iter().filter(|b| b.alive).count()
    }

    /// Update all bullets and remove dead ones.
    pub fn update(&mut self, dt: f64, world_width: f64, world_height: f64) {
        for bullet in &mut self.bullets {
            if bullet.alive {
                bullet.update(dt, world_width, world_height);
            }
        }
        self.bullets.retain(|b| b.alive);
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

    // === Requirement: Bullet Creation ===

    // Scenario: Bullet spawns at ship nose
    #[test]
    fn test_bullet_spawns_at_position() {
        let bullet = Bullet::new(Vec2::new(400.0, 300.0), 0.0);
        assert!(approx_eq(bullet.position.x, 400.0));
        assert!(approx_eq(bullet.position.y, 300.0));
    }

    // Scenario: Bullet direction matches ship rotation
    #[test]
    fn test_bullet_direction_matches_angle() {
        let bullet = Bullet::new(Vec2::new(400.0, 300.0), PI / 2.0);
        // PI/2 means facing down, so velocity should be mostly in +y direction
        assert!(bullet.velocity.y > 0.0);
        assert!(bullet.velocity.x.abs() < EPSILON);
    }

    // === Requirement: Bullet Speed ===

    // Scenario: Bullet travels at fixed speed
    #[test]
    fn test_bullet_fixed_speed() {
        let bullet = Bullet::new(Vec2::new(0.0, 0.0), 0.0);
        assert!(approx_eq(bullet.velocity.magnitude(), BULLET_SPEED));
    }

    // Scenario: Bullet speed is independent of ship velocity
    #[test]
    fn test_bullet_speed_independent() {
        // Bullet is created with just position and angle — no ship velocity involved
        let bullet = Bullet::new(Vec2::new(0.0, 0.0), 0.0);
        assert!(approx_eq(bullet.velocity.x, BULLET_SPEED));
        assert!(approx_eq(bullet.velocity.y, 0.0));
    }

    // === Requirement: Bullet Lifetime (distance-based, matching original Asteroids) ===

    // Scenario: Bullet exists within travel distance
    #[test]
    fn test_bullet_alive_within_travel_distance() {
        // world_width=800, so max range = 640. At 500 u/s, bullet travels ~8.33 units/frame.
        // After 30 frames: ~250 units traveled, well under 640.
        let mut bullet = Bullet::new(Vec2::new(0.0, 0.0), 0.0);
        for _ in 0..30 {
            bullet.update(1.0 / 60.0, 800.0, 600.0);
        }
        assert!(bullet.alive);
        assert!(bullet.distance_traveled < 800.0 * BULLET_RANGE_FRACTION);
    }

    // Scenario: Bullet destroyed after exceeding travel distance
    #[test]
    fn test_bullet_destroyed_after_exceeding_travel_distance() {
        // At 500 u/s and world_width=800, max range=640. Frames needed: 640/(500/60) = ~76.8
        let mut bullet = Bullet::new(Vec2::new(0.0, 0.0), 0.0);
        for _ in 0..77 {
            bullet.update(1.0 / 60.0, 800.0, 600.0);
        }
        assert!(!bullet.alive);
        assert!(bullet.distance_traveled >= 800.0 * BULLET_RANGE_FRACTION);
    }

    // Scenario: Distance threshold scales with world width
    #[test]
    fn test_bullet_distance_scales_with_world_width() {
        // world_width=1000, max range=800. At 500 u/s, frames needed: 800/(500/60) = 96
        // After 95 frames: ~791.67 units, still alive. After 96: 800.0 = threshold, dead.
        let mut bullet = Bullet::new(Vec2::new(0.0, 0.0), 0.0);
        for _ in 0..95 {
            bullet.update(1.0 / 60.0, 1000.0, 600.0);
        }
        assert!(bullet.alive);
        // Compared to world_width=800 (dies at ~77 frames), wider world = longer range
        bullet.update(1.0 / 60.0, 1000.0, 600.0); // 96th frame hits threshold
        assert!(!bullet.alive);
    }

    // === Requirement: Maximum Bullets on Screen ===

    // Scenario: Can fire when under bullet limit
    #[test]
    fn test_fire_under_limit() {
        let mut pool = BulletPool::new();
        for _ in 0..3 {
            assert!(pool.fire(Vec2::new(0.0, 0.0), 0.0));
        }
        assert_eq!(pool.active_count(), 3);
        assert!(pool.fire(Vec2::new(0.0, 0.0), 0.0)); // 4th succeeds
        assert_eq!(pool.active_count(), 4);
    }

    // Scenario: Cannot fire when at bullet limit
    #[test]
    fn test_cannot_fire_at_limit() {
        let mut pool = BulletPool::new();
        for _ in 0..MAX_BULLETS {
            pool.fire(Vec2::new(0.0, 0.0), 0.0);
        }
        assert!(!pool.fire(Vec2::new(0.0, 0.0), 0.0));
    }

    // Scenario: Can fire again after bullet expires
    #[test]
    fn test_fire_after_expiry() {
        let mut pool = BulletPool::new();
        for _ in 0..MAX_BULLETS {
            pool.fire(Vec2::new(0.0, 0.0), 0.0);
        }
        assert_eq!(pool.active_count(), 4);
        // Expire all bullets (need ~77 frames for 800-wide world at 500 u/s)
        for _ in 0..80 {
            pool.update(1.0 / 60.0, 800.0, 600.0);
        }
        assert_eq!(pool.active_count(), 0);
        assert!(pool.fire(Vec2::new(0.0, 0.0), 0.0));
    }

    // === Requirement: Bullet Screen Wrapping ===

    // Scenario: Bullet wraps from right to left
    #[test]
    fn test_bullet_wraps() {
        let mut bullet = Bullet::new(Vec2::new(799.0, 300.0), 0.0);
        bullet.update(1.0 / 60.0, 800.0, 600.0);
        assert!(bullet.position.x < 800.0); // wrapped
    }

    // === Requirement: Bullet Visual Representation ===

    // Scenario: Bullet has small collision radius
    #[test]
    fn test_bullet_radius() {
        assert!(approx_eq(BULLET_RADIUS, 2.0));
    }

    // Scenario: BulletPool implements Default
    #[test]
    fn test_bullet_pool_default() {
        let pool = BulletPool::default();
        assert_eq!(pool.active_count(), 0);
    }
}

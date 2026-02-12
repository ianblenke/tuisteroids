// Bullets capability: projectile creation, lifetime, speed, screen limit

use crate::physics::{self, Vec2};

pub const BULLET_SPEED: f64 = 500.0; // units per second
pub const BULLET_LIFETIME: u32 = 60; // frames at 60 FPS = 1 second
pub const MAX_BULLETS: usize = 4;
pub const BULLET_RADIUS: f64 = 2.0;

pub struct Bullet {
    pub position: Vec2,
    pub velocity: Vec2,
    pub frames_alive: u32,
    pub alive: bool,
}

impl Bullet {
    /// Create a bullet at the given position traveling in the given direction.
    pub fn new(position: Vec2, angle: f64) -> Self {
        let velocity = Vec2::from_angle(angle).scale(BULLET_SPEED);
        Self {
            position,
            velocity,
            frames_alive: 0,
            alive: true,
        }
    }

    /// Update bullet position and lifetime.
    pub fn update(&mut self, dt: f64, world_width: f64, world_height: f64) {
        self.position = physics::integrate_motion(self.position, self.velocity, dt);
        self.position = physics::wrap_position(self.position, world_width, world_height);
        self.frames_alive += 1;
        if self.frames_alive >= BULLET_LIFETIME {
            self.alive = false;
        }
    }
}

/// Manages the collection of active bullets.
pub struct BulletPool {
    pub bullets: Vec<Bullet>,
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

    // === Requirement: Bullet Lifetime ===

    // Scenario: Bullet exists within lifetime
    #[test]
    fn test_bullet_alive_within_lifetime() {
        let mut bullet = Bullet::new(Vec2::new(0.0, 0.0), 0.0);
        for _ in 0..30 {
            bullet.update(1.0 / 60.0, 800.0, 600.0);
        }
        assert!(bullet.alive);
        assert_eq!(bullet.frames_alive, 30);
    }

    // Scenario: Bullet destroyed after lifetime expires
    #[test]
    fn test_bullet_destroyed_after_lifetime() {
        let mut bullet = Bullet::new(Vec2::new(0.0, 0.0), 0.0);
        for _ in 0..BULLET_LIFETIME {
            bullet.update(1.0 / 60.0, 800.0, 600.0);
        }
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
        // Expire all bullets
        for _ in 0..BULLET_LIFETIME {
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
}

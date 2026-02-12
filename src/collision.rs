// Collision capability: circle-circle detection with toroidal distance

use crate::physics::Vec2;

/// Calculate the shortest distance between two points on a toroidal surface.
pub fn toroidal_distance(a: Vec2, b: Vec2, width: f64, height: f64) -> f64 {
    let dx = (a.x - b.x).abs();
    let dy = (a.y - b.y).abs();
    let dx = dx.min(width - dx);
    let dy = dy.min(height - dy);
    (dx * dx + dy * dy).sqrt()
}

/// Calculate the shortest-path direction vector between two points on a toroidal surface.
/// The result's magnitude equals the toroidal distance.
pub fn toroidal_direction(from: Vec2, to: Vec2, width: f64, height: f64) -> Vec2 {
    let mut dx = to.x - from.x;
    let mut dy = to.y - from.y;
    if dx > width / 2.0 {
        dx -= width;
    } else if dx < -width / 2.0 {
        dx += width;
    }
    if dy > height / 2.0 {
        dy -= height;
    } else if dy < -height / 2.0 {
        dy += height;
    }
    Vec2::new(dx, dy)
}

/// Check if two circles collide (distance between centers < sum of radii).
pub fn circles_collide(pos_a: Vec2, radius_a: f64, pos_b: Vec2, radius_b: f64) -> bool {
    let dist = ((pos_a.x - pos_b.x).powi(2) + (pos_a.y - pos_b.y).powi(2)).sqrt();
    dist < radius_a + radius_b
}

/// Check if two circles collide using toroidal distance.
pub fn circles_collide_toroidal(
    pos_a: Vec2,
    radius_a: f64,
    pos_b: Vec2,
    radius_b: f64,
    width: f64,
    height: f64,
) -> bool {
    let dist = toroidal_distance(pos_a, pos_b, width, height);
    dist < radius_a + radius_b
}

/// Result of processing a ship-asteroid collision.
#[derive(Debug, PartialEq)]
pub enum ShipCollisionResult {
    NoCollision,
    ShipDestroyed { lives_remaining: u32 },
    GameOver,
}

/// Check ship-asteroid collision and determine result.
#[allow(clippy::too_many_arguments)]
pub fn check_ship_asteroid_collision(
    ship_pos: Vec2,
    ship_radius: f64,
    ship_lives: u32,
    ship_invulnerable: bool,
    asteroid_pos: Vec2,
    asteroid_radius: f64,
    world_width: f64,
    world_height: f64,
) -> ShipCollisionResult {
    if ship_invulnerable {
        return ShipCollisionResult::NoCollision;
    }

    if !circles_collide_toroidal(
        ship_pos,
        ship_radius,
        asteroid_pos,
        asteroid_radius,
        world_width,
        world_height,
    ) {
        return ShipCollisionResult::NoCollision;
    }

    if ship_lives <= 1 {
        ShipCollisionResult::GameOver
    } else {
        ShipCollisionResult::ShipDestroyed {
            lives_remaining: ship_lives - 1,
        }
    }
}

/// Size of an asteroid for splitting logic.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AsteroidSize {
    Large,
    Medium,
    Small,
}

/// Result of a bullet hitting an asteroid.
#[derive(Debug, PartialEq)]
pub enum BulletAsteroidResult {
    NoCollision,
    AsteroidSplit {
        new_size: AsteroidSize,
        count: usize,
    },
    AsteroidDestroyed,
}

/// Check bullet-asteroid collision and determine split result.
pub fn check_bullet_asteroid_collision(
    bullet_pos: Vec2,
    bullet_radius: f64,
    asteroid_pos: Vec2,
    asteroid_radius: f64,
    asteroid_size: AsteroidSize,
    world_width: f64,
    world_height: f64,
) -> BulletAsteroidResult {
    if !circles_collide_toroidal(
        bullet_pos,
        bullet_radius,
        asteroid_pos,
        asteroid_radius,
        world_width,
        world_height,
    ) {
        return BulletAsteroidResult::NoCollision;
    }

    match asteroid_size {
        AsteroidSize::Large => BulletAsteroidResult::AsteroidSplit {
            new_size: AsteroidSize::Medium,
            count: 2,
        },
        AsteroidSize::Medium => BulletAsteroidResult::AsteroidSplit {
            new_size: AsteroidSize::Small,
            count: 2,
        },
        AsteroidSize::Small => BulletAsteroidResult::AsteroidDestroyed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-10;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < EPSILON
    }

    // === Requirement: Circle-Circle Collision Detection ===

    // Scenario: Overlapping circles collide
    #[test]
    fn test_overlapping_circles_collide() {
        let a = Vec2::new(100.0, 100.0);
        let b = Vec2::new(110.0, 100.0);
        assert!(circles_collide(a, 20.0, b, 20.0));
    }

    // Scenario: Non-overlapping circles do not collide
    #[test]
    fn test_non_overlapping_circles() {
        let a = Vec2::new(100.0, 100.0);
        let b = Vec2::new(200.0, 200.0);
        assert!(!circles_collide(a, 10.0, b, 10.0));
    }

    // Scenario: Touching circles collide
    #[test]
    fn test_touching_circles_collide() {
        let a = Vec2::new(100.0, 100.0);
        let b = Vec2::new(119.0, 100.0);
        assert!(circles_collide(a, 10.0, b, 10.0));
    }

    // === Requirement: Toroidal Distance Calculation ===

    // Scenario: Direct distance is shortest
    #[test]
    fn test_toroidal_direct_distance() {
        let a = Vec2::new(100.0, 100.0);
        let b = Vec2::new(150.0, 100.0);
        let dist = toroidal_distance(a, b, 800.0, 600.0);
        assert!(approx_eq(dist, 50.0));
    }

    // Scenario: Wrapped horizontal distance is shorter
    #[test]
    fn test_toroidal_wrapped_horizontal() {
        let a = Vec2::new(10.0, 300.0);
        let b = Vec2::new(790.0, 300.0);
        let dist = toroidal_distance(a, b, 800.0, 600.0);
        assert!(approx_eq(dist, 20.0));
    }

    // Scenario: Wrapped vertical distance is shorter
    #[test]
    fn test_toroidal_wrapped_vertical() {
        let a = Vec2::new(400.0, 10.0);
        let b = Vec2::new(400.0, 590.0);
        let dist = toroidal_distance(a, b, 800.0, 600.0);
        assert!(approx_eq(dist, 20.0));
    }

    // Scenario: Both axes wrap
    #[test]
    fn test_toroidal_both_axes_wrap() {
        let a = Vec2::new(10.0, 10.0);
        let b = Vec2::new(790.0, 590.0);
        let dist = toroidal_distance(a, b, 800.0, 600.0);
        let expected = (20.0_f64.powi(2) + 20.0_f64.powi(2)).sqrt();
        assert!(approx_eq(dist, expected));
    }

    // === Requirement: Ship-Asteroid Collision ===

    // Scenario: Ship hits asteroid and loses life
    #[test]
    fn test_ship_hits_asteroid_loses_life() {
        let result = check_ship_asteroid_collision(
            Vec2::new(400.0, 300.0),
            12.0,
            3,
            false,
            Vec2::new(405.0, 300.0),
            30.0,
            800.0,
            600.0,
        );
        assert_eq!(
            result,
            ShipCollisionResult::ShipDestroyed { lives_remaining: 2 }
        );
    }

    // Scenario: Invulnerable ship ignores asteroid collision
    #[test]
    fn test_invulnerable_ship_ignores_collision() {
        let result = check_ship_asteroid_collision(
            Vec2::new(400.0, 300.0),
            12.0,
            3,
            true,
            Vec2::new(405.0, 300.0),
            30.0,
            800.0,
            600.0,
        );
        assert_eq!(result, ShipCollisionResult::NoCollision);
    }

    // Scenario: Ship loses last life triggers game over
    #[test]
    fn test_ship_last_life_game_over() {
        let result = check_ship_asteroid_collision(
            Vec2::new(400.0, 300.0),
            12.0,
            1,
            false,
            Vec2::new(405.0, 300.0),
            30.0,
            800.0,
            600.0,
        );
        assert_eq!(result, ShipCollisionResult::GameOver);
    }

    // === Requirement: Bullet-Asteroid Collision ===

    // Scenario: Bullet hits large asteroid
    #[test]
    fn test_bullet_hits_large_asteroid() {
        let result = check_bullet_asteroid_collision(
            Vec2::new(200.0, 200.0),
            2.0,
            Vec2::new(205.0, 200.0),
            30.0,
            AsteroidSize::Large,
            800.0,
            600.0,
        );
        assert_eq!(
            result,
            BulletAsteroidResult::AsteroidSplit {
                new_size: AsteroidSize::Medium,
                count: 2,
            }
        );
    }

    // Scenario: Bullet hits medium asteroid
    #[test]
    fn test_bullet_hits_medium_asteroid() {
        let result = check_bullet_asteroid_collision(
            Vec2::new(200.0, 200.0),
            2.0,
            Vec2::new(205.0, 200.0),
            20.0,
            AsteroidSize::Medium,
            800.0,
            600.0,
        );
        assert_eq!(
            result,
            BulletAsteroidResult::AsteroidSplit {
                new_size: AsteroidSize::Small,
                count: 2,
            }
        );
    }

    // Scenario: Bullet hits small asteroid
    #[test]
    fn test_bullet_hits_small_asteroid() {
        let result = check_bullet_asteroid_collision(
            Vec2::new(200.0, 200.0),
            2.0,
            Vec2::new(205.0, 200.0),
            10.0,
            AsteroidSize::Small,
            800.0,
            600.0,
        );
        assert_eq!(result, BulletAsteroidResult::AsteroidDestroyed);
    }

    // === Requirement: Toroidal Direction Calculation ===

    // Scenario: Direct direction is shortest
    #[test]
    fn test_toroidal_direction_direct() {
        let dir = toroidal_direction(
            Vec2::new(100.0, 100.0),
            Vec2::new(150.0, 100.0),
            800.0,
            600.0,
        );
        assert!(approx_eq(dir.x, 50.0));
        assert!(approx_eq(dir.y, 0.0));
    }

    // Scenario: Wrapped horizontal direction is shorter
    #[test]
    fn test_toroidal_direction_wrapped_horizontal() {
        let dir = toroidal_direction(
            Vec2::new(10.0, 300.0),
            Vec2::new(790.0, 300.0),
            800.0,
            600.0,
        );
        assert!(approx_eq(dir.x, -20.0));
        assert!(approx_eq(dir.y, 0.0));
    }

    // Scenario: Wrapped vertical direction is shorter
    #[test]
    fn test_toroidal_direction_wrapped_vertical() {
        let dir = toroidal_direction(
            Vec2::new(400.0, 10.0),
            Vec2::new(400.0, 590.0),
            800.0,
            600.0,
        );
        assert!(approx_eq(dir.x, 0.0));
        assert!(approx_eq(dir.y, -20.0));
    }

    // Scenario: Both axes wrap
    #[test]
    fn test_toroidal_direction_both_axes_wrap() {
        let dir = toroidal_direction(Vec2::new(10.0, 10.0), Vec2::new(790.0, 590.0), 800.0, 600.0);
        assert!(approx_eq(dir.x, -20.0));
        assert!(approx_eq(dir.y, -20.0));
    }

    // Additional coverage: wrap direction when from > to (negative delta branches)
    #[test]
    fn test_toroidal_direction_negative_wrap() {
        // from=(790,590) to=(10,10): dx=10-790=-780 < -400 → dx+=800=20, dy=10-590=-580 < -300 → dy+=600=20
        let dir = toroidal_direction(Vec2::new(790.0, 590.0), Vec2::new(10.0, 10.0), 800.0, 600.0);
        assert!(approx_eq(dir.x, 20.0));
        assert!(approx_eq(dir.y, 20.0));
    }

    // Scenario: Bullet and asteroid at wrapping boundary
    #[test]
    fn test_bullet_asteroid_wrapping_boundary() {
        let result = check_bullet_asteroid_collision(
            Vec2::new(5.0, 300.0),
            2.0,
            Vec2::new(795.0, 300.0),
            30.0,
            AsteroidSize::Large,
            800.0,
            600.0,
        );
        assert_eq!(
            result,
            BulletAsteroidResult::AsteroidSplit {
                new_size: AsteroidSize::Medium,
                count: 2,
            }
        );
    }
}

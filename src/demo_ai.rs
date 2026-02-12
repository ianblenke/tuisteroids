// Demo AI capability: AI-controlled ship input for attract mode

use crate::asteroids::Asteroid;
use crate::collision;
use crate::input::InputState;
use crate::ship::Ship;
use std::f64::consts::PI;

const ROTATION_DEADZONE: f64 = 0.1; // radians — prevents jittery oscillation
const FIRE_THRESHOLD: f64 = 0.2; // radians — how aligned to fire
const THRUST_THRESHOLD: f64 = 0.5; // radians — how aligned to thrust

/// Generate AI input for the demo ship. Pure function: reads ship/asteroids, returns InputState.
pub fn generate_demo_input(
    ship: &Ship,
    asteroids: &[Asteroid],
    world_width: f64,
    world_height: f64,
) -> InputState {
    if asteroids.is_empty() {
        return InputState::default();
    }

    // Find nearest asteroid using toroidal distance
    let nearest = asteroids
        .iter()
        .min_by(|a, b| {
            let da = collision::toroidal_distance(ship.position, a.position, world_width, world_height);
            let db = collision::toroidal_distance(ship.position, b.position, world_width, world_height);
            da.partial_cmp(&db).unwrap()
        })
        .unwrap();

    // Compute direction to nearest asteroid via shortest toroidal path
    let direction = collision::toroidal_direction(
        ship.position,
        nearest.position,
        world_width,
        world_height,
    );
    let target_angle = direction.y.atan2(direction.x);

    // Signed angle difference, normalized to [-PI, PI]
    let mut angle_diff = target_angle - ship.rotation;
    while angle_diff > PI {
        angle_diff -= 2.0 * PI;
    }
    while angle_diff < -PI {
        angle_diff += 2.0 * PI;
    }

    InputState {
        rotate_left: angle_diff < -ROTATION_DEADZONE,
        rotate_right: angle_diff > ROTATION_DEADZONE,
        thrust: angle_diff.abs() < THRUST_THRESHOLD,
        fire: angle_diff.abs() < FIRE_THRESHOLD,
        quit: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asteroids::{Asteroid, AsteroidSize};
    use crate::physics::Vec2;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn make_asteroid(x: f64, y: f64) -> Asteroid {
        let mut rng = StdRng::seed_from_u64(1);
        Asteroid::new(
            Vec2::new(x, y),
            Vec2::new(0.0, 0.0),
            AsteroidSize::Large,
            &mut rng,
        )
    }

    fn make_ship(x: f64, y: f64, rotation: f64) -> Ship {
        let mut ship = Ship::new(x, y);
        ship.rotation = rotation;
        ship
    }

    // === Requirement: Demo AI Target Selection ===

    // Scenario: AI selects nearest asteroid
    #[test]
    fn test_ai_selects_nearest_asteroid() {
        let ship = make_ship(400.0, 300.0, 0.0);
        let asteroids = vec![
            make_asteroid(100.0, 100.0), // far
            make_asteroid(450.0, 310.0), // near
        ];
        let input = generate_demo_input(&ship, &asteroids, 800.0, 600.0);
        // Nearest asteroid is at (450, 310) — slightly below-right.
        // Angle to it: atan2(10, 50) ≈ 0.197 rad. Ship facing 0.
        // angle_diff ≈ 0.197 > ROTATION_DEADZONE(0.1) → rotate_right
        assert!(input.rotate_right);
        assert!(!input.rotate_left);
    }

    // Scenario: AI handles no asteroids
    #[test]
    fn test_ai_handles_no_asteroids() {
        let ship = make_ship(400.0, 300.0, 0.0);
        let input = generate_demo_input(&ship, &[], 800.0, 600.0);
        assert!(!input.rotate_left);
        assert!(!input.rotate_right);
        assert!(!input.thrust);
        assert!(!input.fire);
        assert!(!input.quit);
    }

    // Scenario: AI uses toroidal distance for target selection
    #[test]
    fn test_ai_uses_toroidal_distance() {
        let ship = make_ship(10.0, 300.0, 0.0);
        let asteroids = vec![
            make_asteroid(200.0, 300.0), // direct distance 190
            make_asteroid(790.0, 300.0), // toroidal distance 20 (wraps left)
        ];
        let input = generate_demo_input(&ship, &asteroids, 800.0, 600.0);
        // Nearest via toroidal: (790, 300). Direction from (10,300) wraps to (-20, 0).
        // target_angle = atan2(0, -20) = PI. Ship facing 0. angle_diff = PI.
        // After normalization: PI > THRUST_THRESHOLD → no thrust
        // PI > FIRE_THRESHOLD → no fire
        // PI > ROTATION_DEADZONE → but normalized to PI, which triggers rotate_left
        // (angle_diff = PI is ambiguous; with normalization it stays at PI > 0 → rotate_right)
        // Actually: angle_diff = PI = PI, not < -ROTATION_DEADZONE, and > ROTATION_DEADZONE
        // So rotate_right = true (ship needs to turn to face the asteroid behind it)
        assert!(!input.thrust); // facing wrong way
        assert!(!input.fire); // facing wrong way
    }

    // === Requirement: Demo AI Rotation ===

    // Scenario: AI rotates right toward target (positive angle difference)
    #[test]
    fn test_ai_rotates_right_toward_target() {
        // Ship facing right (0), asteroid is below (angle ~PI/2)
        let ship = make_ship(400.0, 300.0, 0.0);
        let asteroids = vec![make_asteroid(400.0, 500.0)];
        let input = generate_demo_input(&ship, &asteroids, 800.0, 600.0);
        // target_angle = atan2(200, 0) = PI/2. angle_diff = PI/2 > 0 → rotate_right
        assert!(input.rotate_right);
        assert!(!input.rotate_left);
    }

    // Scenario: AI rotates left toward target (negative angle difference)
    #[test]
    fn test_ai_rotates_left_toward_target() {
        // Ship facing right (0), asteroid is above (angle ~-PI/2)
        let ship = make_ship(400.0, 300.0, 0.0);
        let asteroids = vec![make_asteroid(400.0, 100.0)];
        let input = generate_demo_input(&ship, &asteroids, 800.0, 600.0);
        // target_angle = atan2(-200, 0) = -PI/2. angle_diff = -PI/2 < 0 → rotate_left
        assert!(input.rotate_left);
        assert!(!input.rotate_right);
    }

    // Scenario: AI does not rotate when aligned within deadzone
    #[test]
    fn test_ai_no_rotation_when_aligned() {
        // Ship facing right (0), asteroid is directly right
        let ship = make_ship(400.0, 300.0, 0.0);
        let asteroids = vec![make_asteroid(600.0, 300.0)];
        let input = generate_demo_input(&ship, &asteroids, 800.0, 600.0);
        // target_angle = atan2(0, 200) = 0. angle_diff = 0 within deadzone.
        assert!(!input.rotate_left);
        assert!(!input.rotate_right);
    }

    // === Requirement: Demo AI Firing ===

    // Scenario: AI fires when aligned
    #[test]
    fn test_ai_fires_when_aligned() {
        let ship = make_ship(400.0, 300.0, 0.0);
        let asteroids = vec![make_asteroid(600.0, 300.0)]; // directly ahead
        let input = generate_demo_input(&ship, &asteroids, 800.0, 600.0);
        assert!(input.fire);
    }

    // Scenario: AI does not fire when misaligned
    #[test]
    fn test_ai_no_fire_when_misaligned() {
        let ship = make_ship(400.0, 300.0, 0.0);
        let asteroids = vec![make_asteroid(400.0, 100.0)]; // above, angle_diff = -PI/2
        let input = generate_demo_input(&ship, &asteroids, 800.0, 600.0);
        assert!(!input.fire);
    }

    // === Requirement: Demo AI Thrust ===

    // Scenario: AI thrusts when roughly facing target
    #[test]
    fn test_ai_thrusts_when_roughly_facing() {
        let ship = make_ship(400.0, 300.0, 0.0);
        let asteroids = vec![make_asteroid(600.0, 300.0)]; // directly ahead
        let input = generate_demo_input(&ship, &asteroids, 800.0, 600.0);
        assert!(input.thrust);
    }

    // Scenario: AI does not thrust when facing away
    #[test]
    fn test_ai_no_thrust_when_facing_away() {
        let ship = make_ship(400.0, 300.0, 0.0);
        let asteroids = vec![make_asteroid(400.0, 100.0)]; // above, angle_diff = -PI/2 > THRUST_THRESHOLD
        let input = generate_demo_input(&ship, &asteroids, 800.0, 600.0);
        assert!(!input.thrust);
    }

    // === Requirement: Demo AI Quit Independence ===

    // Scenario: AI never quits
    #[test]
    fn test_ai_never_quits() {
        let ship = make_ship(400.0, 300.0, 0.0);
        let asteroids = vec![make_asteroid(600.0, 300.0)];
        let input = generate_demo_input(&ship, &asteroids, 800.0, 600.0);
        assert!(!input.quit);
    }

    // Edge case: AI with single asteroid
    #[test]
    fn test_ai_single_asteroid() {
        let ship = make_ship(400.0, 300.0, 0.0);
        let asteroids = vec![make_asteroid(500.0, 300.0)];
        let input = generate_demo_input(&ship, &asteroids, 800.0, 600.0);
        // Should target the only asteroid
        assert!(input.fire); // aligned
        assert!(input.thrust); // facing roughly right
    }

    // Edge case: AI wrapping angle calculation (ship at ~PI, target across boundary)
    #[test]
    fn test_ai_wrapping_angle_calculation() {
        // Ship facing left (PI), asteroid directly behind (angle ~0 = right)
        let ship = make_ship(400.0, 300.0, PI);
        let asteroids = vec![make_asteroid(600.0, 300.0)];
        let input = generate_demo_input(&ship, &asteroids, 800.0, 600.0);
        // target_angle = 0, ship.rotation = PI. angle_diff = 0 - PI = -PI.
        // Normalized: -PI stays -PI. -PI < -ROTATION_DEADZONE → rotate_left
        assert!(input.rotate_left || input.rotate_right); // must rotate to face target
        assert!(!input.thrust); // facing wrong way
        assert!(!input.fire); // facing wrong way
    }

    // Edge case: angle_diff > PI triggers normalization (subtract 2*PI)
    #[test]
    fn test_ai_angle_normalization_positive() {
        // Ship rotation = -3.5, target to the right: target_angle = 0
        // angle_diff = 0 - (-3.5) = 3.5 > PI → 3.5 - 2*PI ≈ -2.78 → rotate_left
        let ship = make_ship(400.0, 300.0, -3.5);
        let asteroids = vec![make_asteroid(600.0, 300.0)];
        let input = generate_demo_input(&ship, &asteroids, 800.0, 600.0);
        assert!(input.rotate_left);
    }

    // Edge case: angle_diff < -PI triggers normalization (add 2*PI)
    #[test]
    fn test_ai_angle_normalization_negative() {
        // Ship rotation = 3.5, target to the right: target_angle = 0
        // angle_diff = 0 - 3.5 = -3.5 < -PI → -3.5 + 2*PI ≈ 2.78 → rotate_right
        let ship = make_ship(400.0, 300.0, 3.5);
        let asteroids = vec![make_asteroid(600.0, 300.0)];
        let input = generate_demo_input(&ship, &asteroids, 800.0, 600.0);
        assert!(input.rotate_right);
    }
}

// Asteroids capability: asteroid sizes, splitting, wave system, scoring

use crate::physics::{self, Vec2};
use rand::Rng;
use std::f64::consts::PI;

/// Asteroid size determines radius, point value, and split behavior.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AsteroidSize {
    Large,
    Medium,
    Small,
}

impl AsteroidSize {
    pub fn radius(self) -> f64 {
        match self {
            AsteroidSize::Large => 40.0,
            AsteroidSize::Medium => 20.0,
            AsteroidSize::Small => 10.0,
        }
    }

    pub fn points(self) -> u32 {
        match self {
            AsteroidSize::Large => 20,
            AsteroidSize::Medium => 50,
            AsteroidSize::Small => 100,
        }
    }

    pub fn split_into(self) -> Option<AsteroidSize> {
        match self {
            AsteroidSize::Large => Some(AsteroidSize::Medium),
            AsteroidSize::Medium => Some(AsteroidSize::Small),
            AsteroidSize::Small => None,
        }
    }
}

pub struct Asteroid {
    pub position: Vec2,
    pub velocity: Vec2,
    pub rotation: f64,
    pub angular_velocity: f64,
    pub size: AsteroidSize,
    pub vertices: Vec<Vec2>,
}

impl Asteroid {
    /// Create a new asteroid with random shape.
    pub fn new<R: Rng>(position: Vec2, velocity: Vec2, size: AsteroidSize, rng: &mut R) -> Self {
        let radius = size.radius();
        let num_vertices = rng.gen_range(8..=12);
        let angular_velocity = rng.gen_range(-1.0..1.0);
        let vertices = generate_shape(radius, num_vertices, rng);

        Self {
            position,
            velocity,
            rotation: rng.gen_range(0.0..(2.0 * PI)),
            angular_velocity,
            size,
            vertices,
        }
    }

    /// Create a new asteroid with a specific shape (for deterministic testing).
    pub fn new_with_shape(
        position: Vec2,
        velocity: Vec2,
        size: AsteroidSize,
        angular_velocity: f64,
        vertices: Vec<Vec2>,
    ) -> Self {
        Self {
            position,
            velocity,
            rotation: 0.0,
            angular_velocity,
            size,
            vertices,
        }
    }

    /// Update position and rotation.
    pub fn update(&mut self, dt: f64, world_width: f64, world_height: f64) {
        self.position = physics::integrate_motion(self.position, self.velocity, dt);
        self.position = physics::wrap_position(self.position, world_width, world_height);
        self.rotation = physics::rotate_angle(self.rotation, self.angular_velocity, dt);
    }

    /// Get world-space vertices (rotated and translated).
    pub fn world_vertices(&self) -> Vec<Vec2> {
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();
        self.vertices
            .iter()
            .map(|v| {
                Vec2::new(
                    self.position.x + v.x * cos_r - v.y * sin_r,
                    self.position.y + v.x * sin_r + v.y * cos_r,
                )
            })
            .collect()
    }

    /// Split this asteroid into two children. Returns None for small asteroids.
    pub fn split<R: Rng>(&self, rng: &mut R) -> Option<[Asteroid; 2]> {
        let child_size = self.size.split_into()?;

        let speed = self.velocity.magnitude() * 1.2; // slightly faster
        let base_angle = self.velocity.y.atan2(self.velocity.x);

        let angle1 = base_angle + rng.gen_range(0.3..0.8);
        let angle2 = base_angle - rng.gen_range(0.3..0.8);

        let vel1 = Vec2::from_angle(angle1).scale(speed.max(20.0));
        let vel2 = Vec2::from_angle(angle2).scale(speed.max(20.0));

        let child1 = Asteroid::new(self.position, vel1, child_size, rng);
        let child2 = Asteroid::new(self.position, vel2, child_size, rng);

        Some([child1, child2])
    }
}

/// Generate a random irregular polygon shape for an asteroid.
fn generate_shape<R: Rng>(radius: f64, num_vertices: usize, rng: &mut R) -> Vec<Vec2> {
    let angle_step = 2.0 * PI / num_vertices as f64;
    (0..num_vertices)
        .map(|i| {
            let angle = i as f64 * angle_step;
            let dist = radius * rng.gen_range(0.5..1.2);
            Vec2::new(angle.cos() * dist, angle.sin() * dist)
        })
        .collect()
}

/// Wave system: determines how many large asteroids to spawn.
pub fn wave_asteroid_count(wave: u32) -> u32 {
    wave + 3
}

/// Spawn asteroids for a new wave, away from the ship.
pub fn spawn_wave<R: Rng>(
    wave: u32,
    ship_position: Vec2,
    min_distance: f64,
    world_width: f64,
    world_height: f64,
    rng: &mut R,
) -> Vec<Asteroid> {
    let count = wave_asteroid_count(wave);
    let mut asteroids = Vec::with_capacity(count as usize);

    for _ in 0..count {
        let pos = loop {
            let candidate = Vec2::new(
                rng.gen_range(0.0..world_width),
                rng.gen_range(0.0..world_height),
            );
            let dx = (candidate.x - ship_position.x)
                .abs()
                .min(world_width - (candidate.x - ship_position.x).abs());
            let dy = (candidate.y - ship_position.y)
                .abs()
                .min(world_height - (candidate.y - ship_position.y).abs());
            if (dx * dx + dy * dy).sqrt() >= min_distance {
                break candidate;
            }
        };

        let speed = rng.gen_range(20.0..80.0);
        let angle = rng.gen_range(0.0..(2.0 * PI));
        let velocity = Vec2::from_angle(angle).scale(speed);

        asteroids.push(Asteroid::new(pos, velocity, AsteroidSize::Large, rng));
    }

    asteroids
}

/// Score for destroying an asteroid.
pub fn score_for_size(size: AsteroidSize) -> u32 {
    size.points()
}

/// Calculate total score for fully destroying a large asteroid and all children.
pub fn full_destroy_score() -> u32 {
    // 1 large (20) + 2 medium (50 each) + 4 small (100 each) = 520
    AsteroidSize::Large.points()
        + 2 * AsteroidSize::Medium.points()
        + 4 * AsteroidSize::Small.points()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn test_rng() -> StdRng {
        StdRng::seed_from_u64(42)
    }

    // === Requirement: Asteroid Sizes ===

    // Scenario: Large asteroid properties
    #[test]
    fn test_large_asteroid_properties() {
        assert_eq!(AsteroidSize::Large.radius(), 40.0);
        assert_eq!(AsteroidSize::Large.points(), 20);
    }

    // Scenario: Medium asteroid properties
    #[test]
    fn test_medium_asteroid_properties() {
        assert_eq!(AsteroidSize::Medium.radius(), 20.0);
        assert_eq!(AsteroidSize::Medium.points(), 50);
    }

    // Scenario: Small asteroid properties
    #[test]
    fn test_small_asteroid_properties() {
        assert_eq!(AsteroidSize::Small.radius(), 10.0);
        assert_eq!(AsteroidSize::Small.points(), 100);
    }

    // === Requirement: Asteroid Shape Generation ===

    // Scenario: Asteroid has polygon vertices
    #[test]
    fn test_asteroid_has_vertices() {
        let mut rng = test_rng();
        let a = Asteroid::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 0.0),
            AsteroidSize::Large,
            &mut rng,
        );
        assert!(a.vertices.len() >= 8 && a.vertices.len() <= 12);
    }

    // Scenario: Asteroid vertices vary per instance
    #[test]
    fn test_asteroid_vertices_vary() {
        let mut rng1 = StdRng::seed_from_u64(1);
        let mut rng2 = StdRng::seed_from_u64(2);
        let a1 = Asteroid::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 0.0),
            AsteroidSize::Large,
            &mut rng1,
        );
        let a2 = Asteroid::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 0.0),
            AsteroidSize::Large,
            &mut rng2,
        );
        // At least one vertex should differ
        let differs = a1
            .vertices
            .iter()
            .zip(a2.vertices.iter())
            .any(|(v1, v2)| (v1.x - v2.x).abs() > 0.001 || (v1.y - v2.y).abs() > 0.001);
        assert!(differs);
    }

    // Scenario: Vertices are within radius bounds
    #[test]
    fn test_vertices_within_radius_bounds() {
        let mut rng = test_rng();
        let radius = AsteroidSize::Large.radius();
        let a = Asteroid::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 0.0),
            AsteroidSize::Large,
            &mut rng,
        );
        for v in &a.vertices {
            let dist = v.magnitude();
            assert!(dist >= 0.5 * radius - 0.01, "vertex too close: {}", dist);
            assert!(dist <= 1.2 * radius + 0.01, "vertex too far: {}", dist);
        }
    }

    // === Requirement: Asteroid Splitting ===

    // Scenario: Large asteroid splits into two medium
    #[test]
    fn test_large_splits_into_two_medium() {
        let mut rng = test_rng();
        let a = Asteroid::new(
            Vec2::new(300.0, 300.0),
            Vec2::new(10.0, 5.0),
            AsteroidSize::Large,
            &mut rng,
        );
        let children = a.split(&mut rng).unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].size, AsteroidSize::Medium);
        assert_eq!(children[1].size, AsteroidSize::Medium);
    }

    // Scenario: Medium asteroid splits into two small
    #[test]
    fn test_medium_splits_into_two_small() {
        let mut rng = test_rng();
        let a = Asteroid::new(
            Vec2::new(300.0, 300.0),
            Vec2::new(10.0, 5.0),
            AsteroidSize::Medium,
            &mut rng,
        );
        let children = a.split(&mut rng).unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].size, AsteroidSize::Small);
        assert_eq!(children[1].size, AsteroidSize::Small);
    }

    // Scenario: Small asteroid is destroyed completely
    #[test]
    fn test_small_no_split() {
        let mut rng = test_rng();
        let a = Asteroid::new(
            Vec2::new(300.0, 300.0),
            Vec2::new(10.0, 5.0),
            AsteroidSize::Small,
            &mut rng,
        );
        assert!(a.split(&mut rng).is_none());
    }

    // Scenario: Split asteroids inherit modified velocity
    #[test]
    fn test_split_asteroids_have_velocity() {
        let mut rng = test_rng();
        let a = Asteroid::new(
            Vec2::new(300.0, 300.0),
            Vec2::new(10.0, 5.0),
            AsteroidSize::Large,
            &mut rng,
        );
        let children = a.split(&mut rng).unwrap();
        // Children should have non-zero velocity
        assert!(children[0].velocity.magnitude() > 0.0);
        assert!(children[1].velocity.magnitude() > 0.0);
        // Children should have different velocity directions
        let angle1 = children[0].velocity.y.atan2(children[0].velocity.x);
        let angle2 = children[1].velocity.y.atan2(children[1].velocity.x);
        assert!((angle1 - angle2).abs() > 0.01);
    }

    // === Requirement: Asteroid Movement ===

    // Scenario: Asteroid moves at constant velocity
    #[test]
    fn test_asteroid_constant_velocity() {
        let vel = Vec2::new(30.0, 20.0);
        let mut a = Asteroid::new_with_shape(
            Vec2::new(100.0, 100.0),
            vel,
            AsteroidSize::Large,
            0.0,
            vec![Vec2::new(1.0, 0.0)],
        );
        a.update(1.0 / 60.0, 800.0, 600.0);
        assert!((a.velocity.x - 30.0).abs() < 1e-10);
        assert!((a.velocity.y - 20.0).abs() < 1e-10);
    }

    // Scenario: Asteroid wraps around screen edge
    #[test]
    fn test_asteroid_wraps() {
        let mut a = Asteroid::new_with_shape(
            Vec2::new(799.0, 300.0),
            Vec2::new(300.0, 0.0),
            AsteroidSize::Large,
            0.0,
            vec![Vec2::new(1.0, 0.0)],
        );
        a.update(1.0 / 60.0, 800.0, 600.0);
        assert!(a.position.x < 800.0);
    }

    // Scenario: Asteroid rotates visually
    #[test]
    fn test_asteroid_rotates() {
        let mut a = Asteroid::new_with_shape(
            Vec2::new(100.0, 100.0),
            Vec2::new(0.0, 0.0),
            AsteroidSize::Large,
            0.5,
            vec![Vec2::new(1.0, 0.0)],
        );
        let initial_rotation = a.rotation;
        // Apply 60 steps of 1/60 second each = 1 second
        for _ in 0..60 {
            a.update(1.0 / 60.0, 800.0, 600.0);
        }
        let rotation_change = a.rotation - initial_rotation;
        // Should have rotated by ~0.5 radians (allowing for float imprecision)
        assert!((rotation_change.abs() - 0.5).abs() < 0.02);
    }

    // === Requirement: Wave System ===

    // Scenario: First wave spawns 4 large asteroids
    #[test]
    fn test_wave_1_count() {
        assert_eq!(wave_asteroid_count(1), 4);
    }

    // Scenario: Second wave spawns 5 large asteroids
    #[test]
    fn test_wave_2_count() {
        assert_eq!(wave_asteroid_count(2), 5);
    }

    // Scenario: Asteroids spawn away from ship
    #[test]
    fn test_asteroids_spawn_away_from_ship() {
        let mut rng = test_rng();
        let ship_pos = Vec2::new(400.0, 300.0);
        let asteroids = spawn_wave(1, ship_pos, 150.0, 800.0, 600.0, &mut rng);
        for a in &asteroids {
            let dx = (a.position.x - ship_pos.x)
                .abs()
                .min(800.0 - (a.position.x - ship_pos.x).abs());
            let dy = (a.position.y - ship_pos.y)
                .abs()
                .min(600.0 - (a.position.y - ship_pos.y).abs());
            let dist = (dx * dx + dy * dy).sqrt();
            assert!(dist >= 150.0, "asteroid spawned too close: {}", dist);
        }
        assert_eq!(asteroids.len(), 4);
    }

    // Scenario: New wave after delay when all asteroids destroyed
    // (This is tested in game-loop integration — wave timing is game-level logic)
    #[test]
    fn test_wave_spawns_correct_count() {
        let mut rng = test_rng();
        let asteroids = spawn_wave(3, Vec2::new(400.0, 300.0), 150.0, 800.0, 600.0, &mut rng);
        assert_eq!(asteroids.len(), 6); // wave 3: 3 + 3 = 6
    }

    // === Requirement: Asteroid Scoring ===

    // Scenario: Destroying large asteroid awards 20 points
    #[test]
    fn test_large_score() {
        assert_eq!(score_for_size(AsteroidSize::Large), 20);
    }

    // Scenario: Destroying medium asteroid awards 50 points
    #[test]
    fn test_medium_score() {
        assert_eq!(score_for_size(AsteroidSize::Medium), 50);
    }

    // Scenario: Destroying small asteroid awards 100 points
    #[test]
    fn test_small_score() {
        assert_eq!(score_for_size(AsteroidSize::Small), 100);
    }

    // Scenario: Score accumulates across wave
    #[test]
    fn test_full_destroy_score() {
        assert_eq!(full_destroy_score(), 520);
    }

    // Additional coverage: world_vertices
    #[test]
    fn test_world_vertices_transform() {
        let verts = vec![
            Vec2::new(10.0, 0.0),
            Vec2::new(-5.0, 5.0),
            Vec2::new(-5.0, -5.0),
        ];
        let a = Asteroid::new_with_shape(
            Vec2::new(100.0, 200.0),
            Vec2::new(0.0, 0.0),
            AsteroidSize::Large,
            0.0,
            verts,
        );
        let world = a.world_vertices();
        assert_eq!(world.len(), 3);
        // At rotation 0, cos=1, sin=0, so world vertex = position + local vertex
        assert!((world[0].x - 110.0).abs() < 0.001);
        assert!((world[0].y - 200.0).abs() < 0.001);
    }

    // Additional coverage: spawn_wave retry loop (asteroid too close to ship)
    #[test]
    fn test_spawn_wave_avoids_ship() {
        let mut rng = test_rng();
        // Ship at center, small world — forces retry in spawn logic
        let asteroids = spawn_wave(1, Vec2::new(50.0, 50.0), 10.0, 100.0, 100.0, &mut rng);
        assert_eq!(asteroids.len(), 4);
    }
}

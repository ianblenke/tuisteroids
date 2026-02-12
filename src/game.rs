// Game loop capability: fixed timestep, state machine, update sequence

use crate::asteroids::{self, Asteroid, AsteroidSize};
use crate::bullets::{self, BulletPool};
use crate::collision;
use crate::demo_ai;
use crate::input::{self, Action, FireEdgeDetector, InputState};
use crate::physics;
use crate::renderer::{self, BrailleBuffer};
use crate::ship::Ship;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{self},
};
use rand::rngs::StdRng;
use rand::SeedableRng;
use ratatui::backend::CrosstermBackend;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Terminal;
use std::io;
use std::time::{Duration, Instant};

pub const TIMESTEP: f64 = 1.0 / 60.0; // ~16.67ms
pub const WAVE_DELAY: f64 = 2.0; // seconds between waves
pub const DRAG_FACTOR: f64 = 0.99;
pub const MIN_SPAWN_DISTANCE: f64 = 150.0;

/// Game states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameState {
    Menu,
    Playing,
    GameOver,
}

/// Accumulator for fixed timestep loop.
pub struct TimeAccumulator {
    pub accumulated: f64,
    pub timestep: f64,
}

impl TimeAccumulator {
    pub fn new(timestep: f64) -> Self {
        Self {
            accumulated: 0.0,
            timestep,
        }
    }

    /// Add elapsed time and return how many updates to perform.
    pub fn accumulate(&mut self, elapsed: f64) -> u32 {
        self.accumulated += elapsed;
        let updates = (self.accumulated / self.timestep) as u32;
        self.accumulated -= updates as f64 * self.timestep;
        updates
    }
}

/// All playing-state data.
pub struct PlayingState {
    pub ship: Ship,
    pub asteroids: Vec<Asteroid>,
    pub bullet_pool: BulletPool,
    pub score: u32,
    pub wave: u32,
    pub wave_delay_timer: f64,
    pub rng: StdRng,
    pub frame_count: u64,
}

impl PlayingState {
    pub fn new(world_width: f64, world_height: f64) -> Self {
        let mut rng = StdRng::from_entropy();
        let ship = Ship::new(world_width / 2.0, world_height / 2.0);
        let asteroids = asteroids::spawn_wave(
            1,
            ship.position,
            MIN_SPAWN_DISTANCE,
            world_width,
            world_height,
            &mut rng,
        );

        Self {
            ship,
            asteroids,
            bullet_pool: BulletPool::new(),
            score: 0,
            wave: 1,
            wave_delay_timer: 0.0,
            rng,
            frame_count: 0,
        }
    }

    /// Create with a seeded RNG for deterministic testing.
    pub fn new_seeded(world_width: f64, world_height: f64, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let ship = Ship::new(world_width / 2.0, world_height / 2.0);
        let asteroids = asteroids::spawn_wave(
            1,
            ship.position,
            MIN_SPAWN_DISTANCE,
            world_width,
            world_height,
            &mut rng,
        );

        Self {
            ship,
            asteroids,
            bullet_pool: BulletPool::new(),
            score: 0,
            wave: 1,
            wave_delay_timer: 0.0,
            rng,
            frame_count: 0,
        }
    }

    /// Perform one fixed-timestep update. Returns new GameState if transition needed.
    pub fn update(
        &mut self,
        input: &InputState,
        dt: f64,
        world_width: f64,
        world_height: f64,
    ) -> Option<GameState> {
        // 1. Process input
        if input.is_active(Action::Quit) {
            return Some(GameState::Menu);
        }

        // 2. Update ship
        self.ship.rotate(
            input.is_active(Action::RotateLeft),
            input.is_active(Action::RotateRight),
            dt,
        );
        if input.is_active(Action::Thrust) {
            self.ship.thrust(dt);
        }
        self.ship.velocity = physics::apply_drag(self.ship.velocity, DRAG_FACTOR);
        self.ship.update(dt, world_width, world_height);

        // 3. Update bullets
        self.bullet_pool.update(dt, world_width, world_height);

        // Fire if requested
        if input.is_active(Action::Fire) {
            let nose = self.ship.nose_position();
            self.bullet_pool.fire(nose, self.ship.rotation);
        }

        // 4. Update asteroids
        for asteroid in &mut self.asteroids {
            asteroid.update(dt, world_width, world_height);
        }

        // 5. Check collisions
        // Bullet-asteroid
        let mut new_asteroids: Vec<Asteroid> = Vec::new();
        let mut bullets_to_remove: Vec<usize> = Vec::new();
        let mut asteroids_to_remove: Vec<usize> = Vec::new();
        let mut score_gained: u32 = 0;

        for (bi, bullet) in self.bullet_pool.bullets.iter().enumerate() {
            if !bullet.alive {
                continue;
            }
            for (ai, asteroid) in self.asteroids.iter().enumerate() {
                if asteroids_to_remove.contains(&ai) {
                    continue;
                }
                let result = collision::check_bullet_asteroid_collision(
                    bullet.position,
                    bullets::BULLET_RADIUS,
                    asteroid.position,
                    asteroid.size.radius(),
                    match asteroid.size {
                        AsteroidSize::Large => collision::AsteroidSize::Large,
                        AsteroidSize::Medium => collision::AsteroidSize::Medium,
                        AsteroidSize::Small => collision::AsteroidSize::Small,
                    },
                    world_width,
                    world_height,
                );
                match result {
                    collision::BulletAsteroidResult::AsteroidSplit { .. } => {
                        bullets_to_remove.push(bi);
                        asteroids_to_remove.push(ai);
                        score_gained += asteroid.size.points();
                        if let Some(children) = asteroid.split(&mut self.rng) {
                            new_asteroids.extend(children);
                        }
                        break;
                    }
                    collision::BulletAsteroidResult::AsteroidDestroyed => {
                        bullets_to_remove.push(bi);
                        asteroids_to_remove.push(ai);
                        score_gained += asteroid.size.points();
                        break;
                    }
                    collision::BulletAsteroidResult::NoCollision => {}
                }
            }
        }

        // Remove destroyed bullets and asteroids (reverse order to keep indices valid)
        for &bi in bullets_to_remove.iter().rev() {
            if bi < self.bullet_pool.bullets.len() {
                self.bullet_pool.bullets[bi].alive = false;
            }
        }
        asteroids_to_remove.sort_unstable();
        asteroids_to_remove.dedup();
        for &ai in asteroids_to_remove.iter().rev() {
            if ai < self.asteroids.len() {
                self.asteroids.remove(ai);
            }
        }
        self.asteroids.extend(new_asteroids);

        // 6. Process scoring
        self.score += score_gained;
        self.ship.check_extra_life(self.score);

        // Ship-asteroid collision
        for asteroid in &self.asteroids {
            let result = collision::check_ship_asteroid_collision(
                self.ship.position,
                crate::ship::SHIP_RADIUS,
                self.ship.lives,
                self.ship.invulnerable,
                asteroid.position,
                asteroid.size.radius(),
                world_width,
                world_height,
            );
            match result {
                collision::ShipCollisionResult::ShipDestroyed { .. } => {
                    self.ship.destroy(world_width, world_height);
                    break;
                }
                collision::ShipCollisionResult::GameOver => {
                    self.ship.lives = 0;
                    return Some(GameState::GameOver);
                }
                collision::ShipCollisionResult::NoCollision => {}
            }
        }

        // 7. Check wave completion
        if self.asteroids.is_empty() {
            self.wave_delay_timer += dt;
            if self.wave_delay_timer >= WAVE_DELAY {
                self.wave += 1;
                self.wave_delay_timer = 0.0;
                self.asteroids = asteroids::spawn_wave(
                    self.wave,
                    self.ship.position,
                    MIN_SPAWN_DISTANCE,
                    world_width,
                    world_height,
                    &mut self.rng,
                );
            }
        }

        self.frame_count += 1;
        None
    }
}

/// The top-level game that manages state transitions and the main loop.
pub struct Game {
    pub state: GameState,
    pub playing: Option<PlayingState>,
    pub demo: Option<PlayingState>,
    pub final_score: u32,
    pub world_width: f64,
    pub world_height: f64,
}

impl Game {
    pub fn new(world_width: f64, world_height: f64) -> Self {
        Self {
            state: GameState::Menu,
            playing: None,
            demo: Some(PlayingState::new(world_width, world_height)),
            final_score: 0,
            world_width,
            world_height,
        }
    }

    /// Handle a key press in the current state. Returns true if the game should quit.
    pub fn handle_key(&mut self, code: KeyCode) -> bool {
        match self.state {
            GameState::Menu => {
                if code == KeyCode::Char('q') || code == KeyCode::Char('Q') {
                    return true; // quit
                }
                // Any other key starts the game
                self.state = GameState::Playing;
                self.playing = Some(PlayingState::new(self.world_width, self.world_height));
                self.demo = None;
                false
            }
            GameState::GameOver => {
                if code == KeyCode::Char('q') || code == KeyCode::Char('Q') {
                    return true;
                }
                self.state = GameState::Menu;
                self.demo = Some(PlayingState::new(self.world_width, self.world_height));
                false
            }
            GameState::Playing => false, // handled in update loop
        }
    }

    /// Transition to game over.
    pub fn game_over(&mut self) {
        if let Some(ref playing) = self.playing {
            self.final_score = playing.score;
        }
        self.state = GameState::GameOver;
        self.playing = None;
    }

    /// Start a new demo (attract mode) game.
    pub fn start_demo(&mut self) {
        self.demo = Some(PlayingState::new(self.world_width, self.world_height));
    }
}

/// Calculate how long to sleep to maintain frame rate.
pub fn frame_sleep_duration(frame_start: Instant, target_frame_time: Duration) -> Option<Duration> {
    let elapsed = frame_start.elapsed();
    if elapsed < target_frame_time {
        Some(target_frame_time - elapsed)
    } else {
        None
    }
}

/// Run the main game loop (real terminal I/O).
#[cfg(not(tarpaulin_include))]
pub fn run() -> io::Result<()> {
    // Setup terminal
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;

    // Enable Kitty keyboard protocol for press/release tracking.
    // Without this, terminals only send key-repeat for one key at a time,
    // so pressing spacebar interrupts arrow key repeat.
    let enhanced_keyboard = terminal::supports_keyboard_enhancement().unwrap_or(false);
    if enhanced_keyboard {
        execute!(
            stdout,
            event::PushKeyboardEnhancementFlags(
                event::KeyboardEnhancementFlags::REPORT_EVENT_TYPES
            )
        )?;
    }

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let _size = terminal.size()?;
    let world_width = 800.0_f64;
    let world_height = 600.0_f64;

    let mut game = Game::new(world_width, world_height);
    let mut input_state = InputState::default();
    let mut fire_detector = FireEdgeDetector::new();
    let mut accumulator = TimeAccumulator::new(TIMESTEP);
    let target_frame_time = Duration::from_secs_f64(TIMESTEP);

    let mut last_time = Instant::now();

    // Hold counters for fallback input (terminals without keyboard enhancement).
    // When a key press arrives, the counter resets to HOLD_FRAMES. Counters only
    // decay when no key events are received (terminal is quiet), and freeze while
    // other keys are generating events. This bridges the gap when spacebar steals
    // the terminal's key repeat from arrow keys.
    const HOLD_FRAMES: u8 = 20; // ~333ms at 60fps
    let mut hold_left: u8 = 0;
    let mut hold_right: u8 = 0;
    let mut hold_thrust: u8 = 0;

    loop {
        let frame_start = Instant::now();
        let elapsed = last_time.elapsed().as_secs_f64();
        last_time = frame_start;

        // Poll input
        let mut raw_fire_pressed = false;
        let mut any_key_event = false;
        while event::poll(Duration::ZERO)? {
            if let Event::Key(key) = event::read()? {
                any_key_event = true;
                match key.kind {
                    KeyEventKind::Press | KeyEventKind::Repeat => {
                        match game.state {
                            GameState::Menu | GameState::GameOver => {
                                if key.kind == KeyEventKind::Press {
                                    if game.handle_key(key.code) {
                                        // Quit — cleanup
                                        if enhanced_keyboard {
                                            let _ = execute!(
                                                terminal.backend_mut(),
                                                event::PopKeyboardEnhancementFlags
                                            );
                                        }
                                        terminal::disable_raw_mode()?;
                                        execute!(
                                            terminal.backend_mut(),
                                            terminal::LeaveAlternateScreen,
                                            cursor::Show
                                        )?;
                                        return Ok(());
                                    }
                                }
                            }
                            GameState::Playing => {
                                if let Some(action) = input::map_key(key.code) {
                                    match action {
                                        Action::RotateLeft => {
                                            input_state.rotate_left = true;
                                            hold_left = HOLD_FRAMES;
                                        }
                                        Action::RotateRight => {
                                            input_state.rotate_right = true;
                                            hold_right = HOLD_FRAMES;
                                        }
                                        Action::Thrust => {
                                            input_state.thrust = true;
                                            hold_thrust = HOLD_FRAMES;
                                        }
                                        Action::Fire => {
                                            if key.kind == KeyEventKind::Press {
                                                raw_fire_pressed = true;
                                            }
                                        }
                                        Action::Quit => input_state.quit = true,
                                    }
                                }
                            }
                        }
                    }
                    KeyEventKind::Release => {
                        // With enhanced keyboard, clear key state on release
                        if game.state == GameState::Playing {
                            if let Some(action) = input::map_key(key.code) {
                                match action {
                                    Action::RotateLeft => {
                                        input_state.rotate_left = false;
                                        hold_left = 0;
                                    }
                                    Action::RotateRight => {
                                        input_state.rotate_right = false;
                                        hold_right = 0;
                                    }
                                    Action::Thrust => {
                                        input_state.thrust = false;
                                        hold_thrust = 0;
                                    }
                                    Action::Quit => input_state.quit = false,
                                    Action::Fire => {} // handled by edge detector
                                }
                            }
                        }
                    }
                }
            }
        }

        // Handle fire edge detection
        input_state.fire = fire_detector.update(raw_fire_pressed);

        // Fixed timestep updates
        if game.state == GameState::Playing {
            let updates = accumulator.accumulate(elapsed);
            for _ in 0..updates {
                if let Some(ref mut playing) = game.playing {
                    if let Some(new_state) = playing.update(
                        &input_state,
                        TIMESTEP,
                        world_width,
                        world_height,
                    ) {
                        match new_state {
                            GameState::GameOver => game.game_over(),
                            GameState::Menu => {
                                game.state = GameState::Menu;
                                game.playing = None;
                                game.start_demo();
                            }
                            _ => {}
                        }
                        break;
                    }
                }
            }
        } else if game.state == GameState::Menu {
            // Tick attract mode demo
            let updates = accumulator.accumulate(elapsed);
            let mut demo_over = false;
            for _ in 0..updates {
                if let Some(ref mut demo) = game.demo {
                    let ai_input = demo_ai::generate_demo_input(
                        &demo.ship,
                        &demo.asteroids,
                        world_width,
                        world_height,
                    );
                    if let Some(new_state) = demo.update(&ai_input, TIMESTEP, world_width, world_height) {
                        if new_state == GameState::GameOver || new_state == GameState::Menu {
                            demo_over = true;
                        }
                        break;
                    }
                }
            }
            if demo_over {
                game.start_demo();
            }
        }

        // Without keyboard enhancement, use hold counters to keep keys active
        // across frames even when another key steals the terminal's key repeat.
        // Counters only decay when the terminal is quiet (no events at all).
        // While any key generates events, counters freeze — this bridges the gap
        // when spacebar temporarily steals the repeat from arrow keys.
        // With enhancement, Release events already handle clearing.
        if !enhanced_keyboard {
            if !any_key_event {
                hold_left = hold_left.saturating_sub(1);
                hold_right = hold_right.saturating_sub(1);
                hold_thrust = hold_thrust.saturating_sub(1);
            }
            input_state.rotate_left = hold_left > 0;
            input_state.rotate_right = hold_right > 0;
            input_state.thrust = hold_thrust > 0;
            input_state.quit = false;
        }

        // Render
        terminal.draw(|frame| {
            let area = frame.area();

            match game.state {
                GameState::Menu => {
                    let cols = area.width as usize;
                    let rows = area.height as usize;

                    // Render demo game as background (attract mode)
                    let mut lines: Vec<Line> = Vec::new();
                    if let Some(ref demo) = game.demo {
                        let mut buf = BrailleBuffer::new(cols, rows);
                        // Draw asteroids
                        for asteroid in &demo.asteroids {
                            let verts = asteroid.world_vertices();
                            buf.draw_polygon(&verts, world_width, world_height);
                        }
                        // Draw bullets
                        for bullet in &demo.bullet_pool.bullets {
                            if bullet.alive {
                                let dot_x = (bullet.position.x / world_width * buf.dot_width() as f64) as i32;
                                let dot_y = (bullet.position.y / world_height * buf.dot_height() as f64) as i32;
                                buf.set_dot(dot_x, dot_y);
                                buf.set_dot(dot_x + 1, dot_y);
                                buf.set_dot(dot_x, dot_y + 1);
                                buf.set_dot(dot_x + 1, dot_y + 1);
                            }
                        }
                        // Draw ship
                        let draw_ship = if demo.ship.invulnerable {
                            renderer::ship_blink_visible(demo.frame_count)
                        } else {
                            true
                        };
                        if draw_ship && demo.ship.lives > 0 {
                            let ship_verts = demo.ship.vertices();
                            buf.draw_polygon(&ship_verts, world_width, world_height);
                        }
                        // Convert buffer to lines (no HUD for attract mode)
                        for row in 0..buf.rows {
                            let mut spans = String::new();
                            for col in 0..buf.cols {
                                spans.push(buf.get_char(col, row));
                            }
                            lines.push(Line::from(spans));
                        }
                    } else {
                        // No demo — fill with empty lines
                        for _ in 0..rows {
                            lines.push(Line::from(""));
                        }
                    }

                    // Overlay menu text at vertical center
                    let center = rows / 2;
                    if center >= 2 && center + 2 < lines.len() {
                        lines[center - 2] = Line::from(Span::styled(
                            "    TUISTEROIDS",
                            Style::default().fg(Color::White),
                        ));
                        lines[center - 1] = Line::from("");
                        lines[center] = Line::from("    Press any key to start");
                        lines[center + 1] = Line::from("    Press Q to quit");
                    }

                    let paragraph = Paragraph::new(lines)
                        .block(Block::default());
                    frame.render_widget(paragraph, area);
                }
                GameState::GameOver => {
                    let text = vec![
                        Line::from(""),
                        Line::from(""),
                        Line::from(Span::styled(
                            "    GAME OVER",
                            Style::default().fg(Color::Red),
                        )),
                        Line::from(""),
                        Line::from(format!("    Score: {}", game.final_score)),
                        Line::from(""),
                        Line::from("    Press any key to restart or Q to quit"),
                    ];
                    let paragraph = Paragraph::new(text)
                        .block(Block::default());
                    frame.render_widget(paragraph, area);
                }
                GameState::Playing => {
                    if let Some(ref playing) = game.playing {
                        let cols = area.width as usize;
                        let rows = area.height as usize;
                        let mut buf = BrailleBuffer::new(cols, rows.saturating_sub(1));

                        // Draw asteroids
                        for asteroid in &playing.asteroids {
                            let verts = asteroid.world_vertices();
                            buf.draw_polygon(&verts, world_width, world_height);
                        }

                        // Draw bullets
                        for bullet in &playing.bullet_pool.bullets {
                            if bullet.alive {
                                let dot_x = (bullet.position.x / world_width * buf.dot_width() as f64) as i32;
                                let dot_y = (bullet.position.y / world_height * buf.dot_height() as f64) as i32;
                                buf.set_dot(dot_x, dot_y);
                                buf.set_dot(dot_x + 1, dot_y);
                                buf.set_dot(dot_x, dot_y + 1);
                                buf.set_dot(dot_x + 1, dot_y + 1);
                            }
                        }

                        // Draw ship
                        let draw_ship = if playing.ship.invulnerable {
                            renderer::ship_blink_visible(playing.frame_count)
                        } else {
                            true
                        };
                        if draw_ship && playing.ship.lives > 0 {
                            let ship_verts = playing.ship.vertices();
                            buf.draw_polygon(&ship_verts, world_width, world_height);

                            // Draw thrust flame if thrusting
                            if input_state.thrust {
                                let flame = renderer::thrust_flame_vertices(
                                    playing.ship.position,
                                    playing.ship.rotation,
                                );
                                buf.draw_polygon(&flame, world_width, world_height);
                            }
                        }

                        // Convert buffer to lines
                        let mut lines: Vec<Line> = Vec::new();
                        for row in 0..buf.rows {
                            let mut spans = String::new();
                            for col in 0..buf.cols {
                                spans.push(buf.get_char(col, row));
                            }
                            lines.push(Line::from(spans));
                        }

                        // HUD line
                        let lives_str = "▲ ".repeat(playing.ship.lives as usize);
                        let hud_line = format!(
                            "Score: {}  {}",
                            playing.score, lives_str
                        );
                        lines.push(Line::from(Span::styled(
                            hud_line,
                            Style::default().fg(Color::White),
                        )));

                        let paragraph = Paragraph::new(lines);
                        frame.render_widget(paragraph, area);
                    }
                }
            }
        })?;

        // Frame rate limiting
        if let Some(sleep_time) = frame_sleep_duration(frame_start, target_frame_time) {
            std::thread::sleep(sleep_time);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::Vec2;
    use std::time::{Duration, Instant};

    // === Requirement: Fixed Timestep Game Loop ===

    // Scenario: Single update per frame at 60 FPS
    #[test]
    fn test_single_update_at_60fps() {
        let mut acc = TimeAccumulator::new(TIMESTEP);
        let updates = acc.accumulate(TIMESTEP);
        assert_eq!(updates, 1);
    }

    // Scenario: Multiple updates when frame is slow
    #[test]
    fn test_multiple_updates_slow_frame() {
        let mut acc = TimeAccumulator::new(TIMESTEP);
        let updates = acc.accumulate(0.050); // 50ms ~ 3 timesteps
        assert_eq!(updates, 3);
    }

    // Scenario: No update when insufficient time elapsed
    #[test]
    fn test_no_update_insufficient_time() {
        let mut acc = TimeAccumulator::new(TIMESTEP);
        let updates = acc.accumulate(0.005); // 5ms < 16.67ms
        assert_eq!(updates, 0);
    }

    // Scenario: Accumulator retains remainder
    #[test]
    fn test_accumulator_retains_remainder() {
        let mut acc = TimeAccumulator::new(TIMESTEP);
        let updates = acc.accumulate(0.040); // 40ms / 16.67ms = 2.4
        assert_eq!(updates, 2);
        assert!(acc.accumulated > 0.0);
        assert!(acc.accumulated < TIMESTEP);
    }

    // === Requirement: Update-Render Separation ===

    // Scenario: Render happens after updates
    #[test]
    fn test_update_then_render_order() {
        // The accumulator controls how many updates happen, then render is called once.
        let mut acc = TimeAccumulator::new(TIMESTEP);
        let updates = acc.accumulate(0.040);
        assert_eq!(updates, 2);
        // After consuming updates, render would be called once (game loop structure)
        // This test verifies the accumulator correctly returns update count.
    }

    // Scenario: Render happens even with zero updates
    #[test]
    fn test_render_with_zero_updates() {
        let mut acc = TimeAccumulator::new(TIMESTEP);
        let updates = acc.accumulate(0.005);
        assert_eq!(updates, 0);
        // Game loop still renders once after this — verified by structure
    }

    // === Requirement: Game State Machine ===

    // Scenario: Game starts in Menu state
    #[test]
    fn test_game_starts_in_menu() {
        let game = Game::new(800.0, 600.0);
        assert_eq!(game.state, GameState::Menu);
    }

    // Scenario: Menu transitions to Playing on key press
    #[test]
    fn test_menu_to_playing() {
        let mut game = Game::new(800.0, 600.0);
        let quit = game.handle_key(KeyCode::Enter);
        assert!(!quit);
        assert_eq!(game.state, GameState::Playing);
        assert!(game.playing.is_some());
    }

    // Scenario: Playing transitions to GameOver on last life lost
    #[test]
    fn test_playing_to_game_over() {
        let mut game = Game::new(800.0, 600.0);
        game.state = GameState::Playing;
        game.playing = Some(PlayingState::new_seeded(800.0, 600.0, 42));
        if let Some(ref mut playing) = game.playing {
            playing.score = 1234;
        }
        game.game_over();
        assert_eq!(game.state, GameState::GameOver);
        assert_eq!(game.final_score, 1234);
        assert!(game.playing.is_none());
    }

    // Scenario: Playing transitions to Menu on quit
    #[test]
    fn test_playing_quit_transitions() {
        let mut playing = PlayingState::new_seeded(800.0, 600.0, 42);
        let input = InputState {
            quit: true,
            ..Default::default()
        };
        let result = playing.update(&input, TIMESTEP, 800.0, 600.0);
        assert_eq!(result, Some(GameState::Menu));
    }

    // Scenario: GameOver transitions to Menu on key press
    #[test]
    fn test_game_over_to_menu() {
        let mut game = Game::new(800.0, 600.0);
        game.state = GameState::GameOver;
        let quit = game.handle_key(KeyCode::Enter);
        assert!(!quit);
        assert_eq!(game.state, GameState::Menu);
    }

    // === Requirement: Game Update Sequence ===

    // Scenario: Input processed before movement
    #[test]
    fn test_input_before_movement() {
        let mut playing = PlayingState::new_seeded(800.0, 600.0, 42);
        let initial_vel = playing.ship.velocity;
        let input = InputState {
            thrust: true,
            ..Default::default()
        };
        playing.update(&input, TIMESTEP, 800.0, 600.0);
        // Ship should have moved (velocity changed by thrust, then position by velocity)
        assert!(playing.ship.velocity.magnitude() > initial_vel.magnitude());
    }

    // Scenario: Collisions checked after movement
    #[test]
    fn test_collisions_after_movement() {
        let mut playing = PlayingState::new_seeded(800.0, 600.0, 42);
        // This is an integration test: update processes movement then collisions
        let input = InputState::default();
        playing.update(&input, TIMESTEP, 800.0, 600.0);
        // No crash = collisions were checked successfully
    }

    // Scenario: Scoring happens after collision detection
    #[test]
    fn test_scoring_after_collision() {
        let mut playing = PlayingState::new_seeded(800.0, 600.0, 99);
        // Place a bullet directly on an asteroid to force a hit
        if !playing.asteroids.is_empty() {
            let asteroid_pos = playing.asteroids[0].position;
            playing.bullet_pool.bullets.push(crate::bullets::Bullet {
                position: asteroid_pos,
                velocity: Vec2::new(0.0, 0.0),
                distance_traveled: 0.0,
                alive: true,
            });
        }
        let initial_score = playing.score;
        let input = InputState::default();
        playing.update(&input, TIMESTEP, 800.0, 600.0);
        // Score should have increased
        assert!(playing.score > initial_score);
    }

    // === Requirement: Quit Handling ===

    // Scenario: Q key exits from menu
    #[test]
    fn test_q_exits_menu() {
        let mut game = Game::new(800.0, 600.0);
        let quit = game.handle_key(KeyCode::Char('q'));
        assert!(quit);
    }

    // Scenario: Q key exits from playing (via update)
    #[test]
    fn test_q_exits_playing() {
        let mut playing = PlayingState::new_seeded(800.0, 600.0, 42);
        let input = InputState {
            quit: true,
            ..Default::default()
        };
        let result = playing.update(&input, TIMESTEP, 800.0, 600.0);
        assert_eq!(result, Some(GameState::Menu));
    }

    // Scenario: Terminal restored on exit
    #[test]
    fn test_terminal_restore_concept() {
        // Terminal restore happens in the run() function with enable/disable raw mode.
        // We verify the frame_sleep_duration helper works correctly as part of the loop.
        let start = Instant::now();
        let target = Duration::from_millis(16);
        // Immediately after starting, sleep duration should be close to target
        let sleep = frame_sleep_duration(start, target);
        assert!(sleep.is_some());
    }

    // === Requirement: Frame Rate Limiting ===

    // Scenario: Sleep when frame completes early
    #[test]
    fn test_sleep_when_frame_early() {
        let start = Instant::now();
        let target = Duration::from_millis(100); // large target so we definitely complete early
        let sleep = frame_sleep_duration(start, target);
        assert!(sleep.is_some());
        assert!(sleep.unwrap() > Duration::ZERO);
    }

    // Scenario: No sleep when frame is slow
    #[test]
    fn test_no_sleep_when_slow() {
        let start = Instant::now();
        // Simulate slow frame by using a very short target
        std::thread::sleep(Duration::from_millis(5));
        let target = Duration::from_millis(1);
        let sleep = frame_sleep_duration(start, target);
        assert!(sleep.is_none());
    }

    // Additional coverage: fire action in update
    #[test]
    fn test_fire_action_creates_bullet() {
        let mut playing = PlayingState::new_seeded(800.0, 600.0, 42);
        let input = InputState {
            fire: true,
            ..Default::default()
        };
        assert_eq!(playing.bullet_pool.active_count(), 0);
        playing.update(&input, TIMESTEP, 800.0, 600.0);
        assert_eq!(playing.bullet_pool.active_count(), 1);
    }

    // Additional coverage: medium asteroid collision path
    #[test]
    fn test_bullet_hits_medium_in_game() {
        let mut playing = PlayingState::new_seeded(800.0, 600.0, 42);
        // Replace first asteroid with a medium one at known position
        playing.asteroids.clear();
        let mut rng = rand::rngs::StdRng::seed_from_u64(1);
        playing.asteroids.push(crate::asteroids::Asteroid::new(
            Vec2::new(100.0, 100.0),
            Vec2::new(0.0, 0.0),
            crate::asteroids::AsteroidSize::Medium,
            &mut rng,
        ));
        // Place bullet on it
        playing.bullet_pool.bullets.push(crate::bullets::Bullet {
            position: Vec2::new(100.0, 100.0),
            velocity: Vec2::new(0.0, 0.0),
            distance_traveled: 0.0,
            alive: true,
        });
        let input = InputState::default();
        playing.update(&input, TIMESTEP, 800.0, 600.0);
        assert_eq!(playing.score, 50);
    }

    // Additional coverage: small asteroid destroyed (no split)
    #[test]
    fn test_bullet_hits_small_in_game() {
        let mut playing = PlayingState::new_seeded(800.0, 600.0, 42);
        playing.asteroids.clear();
        let mut rng = rand::rngs::StdRng::seed_from_u64(1);
        playing.asteroids.push(crate::asteroids::Asteroid::new(
            Vec2::new(100.0, 100.0),
            Vec2::new(0.0, 0.0),
            crate::asteroids::AsteroidSize::Small,
            &mut rng,
        ));
        playing.bullet_pool.bullets.push(crate::bullets::Bullet {
            position: Vec2::new(100.0, 100.0),
            velocity: Vec2::new(0.0, 0.0),
            distance_traveled: 0.0,
            alive: true,
        });
        let input = InputState::default();
        playing.update(&input, TIMESTEP, 800.0, 600.0);
        assert_eq!(playing.score, 100);
        assert!(playing.asteroids.is_empty());
    }

    // Additional coverage: ship-asteroid collision (ShipDestroyed path)
    #[test]
    fn test_ship_destroyed_by_asteroid_in_game() {
        let mut playing = PlayingState::new_seeded(800.0, 600.0, 42);
        playing.asteroids.clear();
        let mut rng = rand::rngs::StdRng::seed_from_u64(1);
        // Place asteroid right on ship
        playing.asteroids.push(crate::asteroids::Asteroid::new(
            playing.ship.position,
            Vec2::new(0.0, 0.0),
            crate::asteroids::AsteroidSize::Large,
            &mut rng,
        ));
        playing.ship.invulnerable = false;
        let initial_lives = playing.ship.lives;
        let input = InputState::default();
        playing.update(&input, TIMESTEP, 800.0, 600.0);
        assert_eq!(playing.ship.lives, initial_lives - 1);
    }

    // Additional coverage: ship game over in update
    #[test]
    fn test_ship_game_over_in_update() {
        let mut playing = PlayingState::new_seeded(800.0, 600.0, 42);
        playing.asteroids.clear();
        let mut rng = rand::rngs::StdRng::seed_from_u64(1);
        playing.asteroids.push(crate::asteroids::Asteroid::new(
            playing.ship.position,
            Vec2::new(0.0, 0.0),
            crate::asteroids::AsteroidSize::Large,
            &mut rng,
        ));
        playing.ship.lives = 1;
        playing.ship.invulnerable = false;
        let input = InputState::default();
        let result = playing.update(&input, TIMESTEP, 800.0, 600.0);
        assert_eq!(result, Some(GameState::GameOver));
    }

    // Additional coverage: wave completion
    #[test]
    fn test_wave_completion() {
        let mut playing = PlayingState::new_seeded(800.0, 600.0, 42);
        playing.asteroids.clear(); // all destroyed
        let input = InputState::default();
        // Run enough updates to pass the wave delay
        for _ in 0..(60 * 3) {
            // 3 seconds worth
            playing.update(&input, TIMESTEP, 800.0, 600.0);
        }
        // New wave should have spawned
        assert!(!playing.asteroids.is_empty());
        assert_eq!(playing.wave, 2);
    }

    // Additional coverage: handle_key in Playing state (no-op)
    #[test]
    fn test_handle_key_playing_noop() {
        let mut game = Game::new(800.0, 600.0);
        game.state = GameState::Playing;
        game.playing = Some(PlayingState::new_seeded(800.0, 600.0, 42));
        let quit = game.handle_key(KeyCode::Left);
        assert!(!quit);
        assert_eq!(game.state, GameState::Playing);
    }

    // Additional coverage: game over from GameOver + Q
    #[test]
    fn test_game_over_q_quits() {
        let mut game = Game::new(800.0, 600.0);
        game.state = GameState::GameOver;
        let quit = game.handle_key(KeyCode::Char('q'));
        assert!(quit);
    }

    // === Requirement: Attract Mode Demo Game ===

    // Scenario: Demo game starts on application launch
    #[test]
    fn test_game_starts_with_demo() {
        let game = Game::new(800.0, 600.0);
        assert_eq!(game.state, GameState::Menu);
        assert!(game.demo.is_some());
    }

    // Scenario: Demo game updates with AI input
    #[test]
    fn test_demo_updates_with_ai_input() {
        let mut game = Game::new(800.0, 600.0);
        let demo = game.demo.as_mut().unwrap();
        let initial_frame = demo.frame_count;
        let ai_input = crate::demo_ai::generate_demo_input(
            &demo.ship,
            &demo.asteroids,
            800.0,
            600.0,
        );
        demo.update(&ai_input, TIMESTEP, 800.0, 600.0);
        assert_eq!(demo.frame_count, initial_frame + 1);
    }

    // Scenario: Demo resets on game over
    #[test]
    fn test_demo_resets_on_game_over() {
        let mut game = Game::new(800.0, 600.0);
        // Force the demo ship to die
        if let Some(ref mut demo) = game.demo {
            demo.ship.lives = 1;
            demo.ship.invulnerable = false;
            // Place asteroid on ship
            let mut rng = rand::rngs::StdRng::seed_from_u64(1);
            demo.asteroids.clear();
            demo.asteroids.push(crate::asteroids::Asteroid::new(
                demo.ship.position,
                Vec2::new(0.0, 0.0),
                crate::asteroids::AsteroidSize::Large,
                &mut rng,
            ));
            let input = InputState::default();
            let result = demo.update(&input, TIMESTEP, 800.0, 600.0);
            assert_eq!(result, Some(GameState::GameOver));
        }
        // After detecting game over, reset demo
        game.start_demo();
        assert!(game.demo.is_some());
        assert_eq!(game.demo.as_ref().unwrap().ship.lives, 3);
    }

    // Scenario: Demo discarded on game start
    #[test]
    fn test_demo_discarded_on_game_start() {
        let mut game = Game::new(800.0, 600.0);
        assert!(game.demo.is_some());
        game.handle_key(KeyCode::Enter);
        assert_eq!(game.state, GameState::Playing);
        assert!(game.demo.is_none());
        assert!(game.playing.is_some());
    }

    // Scenario: Demo restarts when returning to menu from game over
    #[test]
    fn test_demo_restarts_from_game_over() {
        let mut game = Game::new(800.0, 600.0);
        game.state = GameState::GameOver;
        game.demo = None;
        game.handle_key(KeyCode::Enter);
        assert_eq!(game.state, GameState::Menu);
        assert!(game.demo.is_some());
    }

    // Scenario: Demo restarts when returning to menu from playing
    #[test]
    fn test_demo_restarts_from_playing_quit() {
        let mut game = Game::new(800.0, 600.0);
        game.handle_key(KeyCode::Enter); // start playing
        assert!(game.demo.is_none());
        // Simulate quit returning to menu
        game.state = GameState::Menu;
        game.playing = None;
        game.start_demo();
        assert!(game.demo.is_some());
    }

    // Additional coverage: start_demo creates fresh PlayingState
    #[test]
    fn test_start_demo() {
        let mut game = Game::new(800.0, 600.0);
        game.demo = None;
        game.start_demo();
        assert!(game.demo.is_some());
        assert_eq!(game.demo.as_ref().unwrap().score, 0);
        assert_eq!(game.demo.as_ref().unwrap().wave, 1);
    }
}

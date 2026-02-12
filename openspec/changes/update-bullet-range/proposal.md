# Change: Update bullet range to match original Asteroids

## Why
The current bullet lifetime is time-based (60 frames = 1.0 seconds), causing bullets to travel ~62% of screen width before expiring. In the original 1979 Atari Asteroids, bullets traveled approximately 80% of the screen width using a distance-based lifetime. This change switches to distance-based bullet expiry to match the original game's feel.

## What Changes
- Modify `bullets` capability: replace frame-based bullet lifetime with distance-based lifetime
- Bullet expires after traveling 80% of `world_width` (640 units in 800-wide world)
- Track cumulative distance traveled instead of frames alive

## Impact
- Affected specs: `bullets`
- Affected code: `src/bullets.rs` (constants, `Bullet` struct, `update` method)

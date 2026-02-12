## ADDED Requirements

### Requirement: Toroidal Direction Calculation
The system SHALL compute the shortest-path direction vector between two points on a toroidal surface. The direction vector's magnitude SHALL equal the toroidal distance. Each axis SHALL independently choose the shorter path (direct or wrapped).

#### Scenario: Direct direction is shortest
- **GIVEN** points at (100, 100) and (150, 100) in an 800x600 world
- **WHEN** toroidal direction is computed
- **THEN** the result SHALL be (50, 0)

#### Scenario: Wrapped horizontal direction is shorter
- **GIVEN** points at (10, 300) and (790, 300) in an 800x600 world
- **WHEN** toroidal direction is computed
- **THEN** the result SHALL be (-20, 0) (wrapping left is shorter)

#### Scenario: Wrapped vertical direction is shorter
- **GIVEN** points at (400, 10) and (400, 590) in an 800x600 world
- **WHEN** toroidal direction is computed
- **THEN** the result SHALL be (0, -20) (wrapping up is shorter)

#### Scenario: Both axes wrap
- **GIVEN** points at (10, 10) and (790, 590) in an 800x600 world
- **WHEN** toroidal direction is computed
- **THEN** the result SHALL be approximately (-20, -20)

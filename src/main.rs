use macroquad::prelude::*;

const PAW_ACCELERATION: f32 = 5.0;
const PAW_FRICTION: f32 = -0.2;
const PAW_SHAPE: Vec2 = Vec2 {
    x: 20.0 / 1.5,
    y: 30.0 / 1.5,
};

const BALL_SHAPE: Vec2 = Vec2 { x: 10.0, y: 10.0 };
const BASE_BALL_VELOCITY: f32 = 0.4;

const GAME_SHAPE: Vec2 = Vec2 { x: 100.0, y: 100.0 };

enum TranslateType {
    Normal,
    JustScale,
}

struct GameArea {
    rect: Rect,
    texture: Texture2D,
}

impl GameArea {
    fn new(texture: Texture2D) -> Self {
        Self {
            rect: Rect::default(),
            texture,
        }
    }

    fn update(&mut self) {
        let screen_size = Vec2 {
            x: screen_width(),
            y: screen_height(),
        };
        let min_axis = screen_size.min_element();
        self.rect.x = screen_size.x / 2.0 - min_axis / 2.0;
        self.rect.y = screen_size.y / 2.0 - min_axis / 2.0;
        self.rect.w = min_axis;
        self.rect.h = min_axis;
    }

    fn draw(&self) {
        draw_texture_ex(
            &self.texture,
            self.rect.x,
            self.rect.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2 {
                    x: self.rect.w,
                    y: self.rect.h,
                }),
                ..Default::default()
            },
        );
    }

    fn game_to_screen(&self, game_point: Vec2, translate_type: TranslateType) -> Vec2 {
        let current_shape = Vec2 {
            x: self.rect.w,
            y: self.rect.h,
        };
        let offset = Vec2 {
            x: self.rect.x,
            y: self.rect.y,
        };
        match translate_type {
            TranslateType::Normal => ((current_shape / GAME_SHAPE) * game_point) + offset,
            TranslateType::JustScale => (current_shape / GAME_SHAPE) * game_point,
        }
    }

    fn screen_to_game(&self, screen_point: Vec2) -> Vec2 {
        let current_shape = Vec2 {
            x: self.rect.w,
            y: self.rect.h,
        };
        let offset = Vec2 {
            x: self.rect.x,
            y: self.rect.y,
        };
        ((screen_point - offset) / current_shape) * GAME_SHAPE
    }
}

enum PawSide {
    Left,
    Right,
}

struct Paw {
    rect: Rect,
    velocity: Vec2,
    paw_side: PawSide,
    texture: Texture2D,
}

impl Paw {
    fn new(texture: Texture2D, paw_side: PawSide) -> Self {
        Self {
            rect: Rect {
                x: match paw_side {
                    PawSide::Left => 25.0,
                    PawSide::Right => 75.0 - PAW_SHAPE.x,
                },
                y: GAME_SHAPE.y - PAW_SHAPE.y,
                w: PAW_SHAPE.x,
                h: PAW_SHAPE.y,
            },
            velocity: Vec2::ZERO,
            paw_side,
            texture,
        }
    }

    fn update(&mut self, game_area: &GameArea) {
        // Get all touch locations in game units
        let mut touches = touches()
            .iter()
            .map(|point| game_area.screen_to_game(point.position))
            .collect::<Vec<Vec2>>();

        // Keep only the touches that should apply to this paw
        touches.retain(|touch| {
            touch.x > 0.0
                && touch.x < GAME_SHAPE.x
                && match self.paw_side {
                    PawSide::Left => touch.x < GAME_SHAPE.x / 2.0,
                    PawSide::Right => touch.x > GAME_SHAPE.x / 2.0,
                }
        });

        // Apply acceleration in the direction of the closest touch
        let mut paw_acceleration: f32 = 0.0;
        let mut smallest_distance: f32 = f32::INFINITY;
        for touch in touches {
            let distance = (touch.x - self.rect.center().x).abs();
            if distance < smallest_distance {
                smallest_distance = distance;
                if touch.x > self.rect.center().x {
                    paw_acceleration = PAW_ACCELERATION / (1.0 / (distance / GAME_SHAPE.x));
                }
                if touch.x < self.rect.center().x {
                    paw_acceleration = -PAW_ACCELERATION / (1.0 / (distance / GAME_SHAPE.x));
                }
            }
        }
        paw_acceleration += self.velocity.x * PAW_FRICTION;
        self.velocity += paw_acceleration;
        self.rect.x += self.velocity.x + 0.5 * paw_acceleration;

        // Clamp the paw's movement so it stays in the area it should
        match self.paw_side {
            PawSide::Left => {
                self.rect.x = self.rect.x.clamp(0.0, (GAME_SHAPE.x / 2.0) - self.rect.w)
            }
            PawSide::Right => {
                self.rect.x = self
                    .rect
                    .x
                    .clamp(GAME_SHAPE.x / 2.0, GAME_SHAPE.x - self.rect.w)
            }
        };
    }

    fn draw(&self, game_area: &GameArea) {
        let screen_size = game_area.game_to_screen(self.rect.size(), TranslateType::JustScale);
        let screen_position = game_area.game_to_screen(self.rect.point(), TranslateType::Normal);

        draw_texture_ex(
            &self.texture,
            screen_position.x,
            screen_position.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2 {
                    x: screen_size.x,
                    y: screen_size.y,
                }),
                ..Default::default()
            },
        );
    }
}

struct Ball {
    rect: Rect,
    velocity: Vec2,
    texture: Texture2D,
}

impl Ball {
    fn new(texture: Texture2D) -> Self {
        Self {
            rect: Rect {
                x: GAME_SHAPE.x / 2.0 - BALL_SHAPE.x / 2.0,
                y: GAME_SHAPE.y / 2.0 - BALL_SHAPE.y / 2.0,
                w: BALL_SHAPE.x,
                h: BALL_SHAPE.y,
            },
            velocity: Vec2 {
                x: BASE_BALL_VELOCITY,
                y: BASE_BALL_VELOCITY,
            },
            texture,
        }
    }

    fn update(&mut self, paw_locations: Vec<Rect>, scores: &mut Scores) {
        // calculate ball velocity
        let ball_velocity =
            BASE_BALL_VELOCITY + BASE_BALL_VELOCITY * ((scores.score + 1) as f32 / 100.0);
        // Check for collision with walls
        if self.rect.x < 0.0 {
            self.velocity.x = ball_velocity;
            scores.score += 1;
        }
        if (self.rect.x + self.rect.w) > GAME_SHAPE.x {
            self.velocity.x = -ball_velocity;
            scores.score += 1;
        }
        if self.rect.y < 0.0 {
            self.velocity.y = ball_velocity;
            scores.score += 1;
        }
        // Check for collision with paws
        for paw_location in paw_locations {
            if self.rect.contains(paw_location.center()) {
                self.velocity.y = -ball_velocity;
                scores.score += 1;
            }
        }
        // Check end-game
        if self.rect.y > GAME_SHAPE.y {
            self.rect.x = GAME_SHAPE.x / 2.0 - BALL_SHAPE.x / 2.0;
            self.rect.y = GAME_SHAPE.y / 2.0 - BALL_SHAPE.y / 2.0;
            self.velocity = Vec2 {
                x: BASE_BALL_VELOCITY,
                y: BASE_BALL_VELOCITY,
            };
            scores.score = 0;
        }

        // Update position
        self.rect.x += self.velocity.x;
        self.rect.y += self.velocity.y;
    }

    fn draw(&self, game_area: &GameArea) {
        let screen_size = game_area.game_to_screen(self.rect.size(), TranslateType::JustScale);
        let screen_position = game_area.game_to_screen(self.rect.point(), TranslateType::Normal);

        draw_texture_ex(
            &self.texture,
            screen_position.x,
            screen_position.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2 {
                    x: screen_size.x,
                    y: screen_size.y,
                }),
                ..Default::default()
            },
        );
    }
}

struct Scores {
    score: u32,
    best_score: u32,
}

impl Scores {
    fn new() -> Self {
        Self {
            score: 0,
            best_score: 0,
        }
    }

    fn update(&mut self) {
        if self.score > self.best_score {
            self.best_score = self.score
        }
    }

    fn draw(&self, game_area: &GameArea) {
        let score_text_area =
            game_area.game_to_screen(Vec2 { x: 5.0, y: 10.0 }, TranslateType::Normal);
        let text_size =
            game_area.game_to_screen(Vec2 { x: 10.0, y: 10.0 }, TranslateType::JustScale);
        draw_text(
            &format!("Score: {}", self.score),
            score_text_area.x,
            score_text_area.y,
            text_size.x,
            BLACK,
        );
        let best_score_text_area =
            game_area.game_to_screen(Vec2 { x: 5.0, y: 17.5 }, TranslateType::Normal);
        draw_text(
            &format!("Best Score: {}", self.best_score),
            best_score_text_area.x,
            best_score_text_area.y,
            text_size.x,
            BLACK,
        );
    }
}

#[macroquad::main("Cat Ball Wow!")]
async fn main() {
    // Load textures
    let ball_texture: Texture2D = load_texture("assets/ball.png").await.unwrap();
    ball_texture.set_filter(FilterMode::Linear);
    let background_texture: Texture2D = load_texture("assets/background.png").await.unwrap();
    background_texture.set_filter(FilterMode::Linear);
    let left_paw_texture: Texture2D = load_texture("assets/paw_left.png").await.unwrap();
    left_paw_texture.set_filter(FilterMode::Linear);
    let right_paw_texture: Texture2D = load_texture("assets/paw_right.png").await.unwrap();
    right_paw_texture.set_filter(FilterMode::Linear);

    // Create game objects
    let mut game_area = GameArea::new(background_texture);
    let mut left_paw = Paw::new(left_paw_texture, PawSide::Left);
    let mut right_paw = Paw::new(right_paw_texture, PawSide::Right);
    let mut ball = Ball::new(ball_texture);
    let mut scores = Scores::new();

    loop {
        clear_background(PINK);

        game_area.update();
        game_area.draw();

        left_paw.update(&game_area);
        right_paw.update(&game_area);
        left_paw.draw(&game_area);
        right_paw.draw(&game_area);

        let paw_locations = vec![left_paw.rect, right_paw.rect];
        ball.update(paw_locations, &mut scores);
        ball.draw(&game_area);

        scores.update();
        scores.draw(&game_area);

        next_frame().await
    }
}

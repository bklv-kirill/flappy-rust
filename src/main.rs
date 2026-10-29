use macroquad::prelude::*;

const GRAVITY: f32 = 1100.0;
const FLAP_VELOCITY: f32 = -350.0;

const HERO_DRAW_WIDTH: f32 = 56.0;
const BACKGROUND_COLOR: Color = Color::from_rgba(24, 28, 40, 255);
const BACKGROUND_SPEED: f32 = 35.0;
const TEXT_COLOR: Color = WHITE;

const HERO_RADIUS: f32 = 16.0;
const HERO_X: f32 = 120.0;
const HERO_Y: f32 = 360.0;
const HERO_UP_ANGLE: f32 = -0.35;
const HERO_DOWN_ANGLE: f32 = 0.70;
const HERO_TILT_RESPONSE: f32 = 12.0;

const PIPE_WIDTH: f32 = 80.0;
const PIPE_MARGIN: f32 = 100.0;
const PIPE_COUNT: usize = 3;
const PIPE_SPACING: f32 = 280.0;
const PIPE_GAP: f32 = 200.0;
const PIPE_SPEED: f32 = 200.0;
const PIPE_CAP_SOURCE: Rect = Rect::new(164.0, 0.0, 397.0, 170.0);
const PIPE_BODY_SOURCE: Rect = Rect::new(179.0, 180.0, 367.0, 1992.0);

#[derive(Clone, Copy, PartialEq)]
enum GameState {
    Ready,
    Playing,
    GameOver,
}

struct Game {
    state: GameState,
    score: u32,

    timer: f32,
    timer_interval: f32,

    hero: Hero,

    gap_between_pipes: f32,
    pipes: Vec<PipePair>,
    pipe_speed: f32,

    background_offset: f32,
}

impl Game {
    fn new(hero: Hero) -> Self {
        Self {
            state: GameState::Ready,
            score: 0,

            timer: 0.0,
            timer_interval: 0.5,

            hero,

            gap_between_pipes: PIPE_GAP,
            pipes: create_pipes(PIPE_GAP),
            pipe_speed: PIPE_SPEED,

            background_offset: 0.0,
        }
    }

    fn reset(&mut self) {
        self.state = GameState::Ready;
        self.score = 0;

        self.hero.y = HERO_Y;
        self.hero.velocity_y = 0.0;
        self.hero.angle = 0.0;

        self.gap_between_pipes = PIPE_GAP;
        self.pipes = create_pipes(self.gap_between_pipes);
        self.pipe_speed = PIPE_SPEED;

        self.background_offset = 0.0;
    }
}

struct PipePair {
    x: f32,
    gap_y: f32,
    gap_between: f32,
    scored: bool,
}

struct Hero {
    y: f32,
    velocity_y: f32,
    size: Vec2,
    angle: f32,
}

impl PipePair {
    fn new(x: f32, gap_between: f32) -> Self {
        let half_gap: f32 = gap_between / 2.0;

        let min_y: f32 = PIPE_MARGIN + half_gap;
        let max_y: f32 = screen_height() - min_y;

        Self {
            x,
            gap_y: rand::gen_range(min_y, max_y),
            gap_between,
            scored: false,
        }
    }

    fn rectangles(&self) -> (Rect, Rect) {
        let gap_top: f32 = self.gap_y - self.gap_between / 2.0;
        let gap_bottom: f32 = self.gap_y + self.gap_between / 2.0;

        let top: Rect = Rect::new(self.x, 0.0, PIPE_WIDTH, gap_top);
        let bottom: Rect = Rect::new(self.x, gap_bottom, PIPE_WIDTH, screen_height() - gap_bottom);

        (top, bottom)
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: String::from("Flappy Rust"),
        window_width: 480,
        window_height: 720,
        window_resizable: false,
        ..Default::default()
    }
}

fn create_pipes(gap_between: f32) -> Vec<PipePair> {
    (0..PIPE_COUNT)
        .map(|i| {
            let x: f32 = screen_width() + (i as f32) * PIPE_SPACING;

            PipePair::new(x, gap_between)
        })
        .collect()
}

fn draw_background(texture: &Texture2D, size: Vec2, offset: f32) {
    let x: f32 = (screen_width() - size.x) / 2.0 - offset;
    let y: f32 = (screen_height() - size.y) / 2.0;

    for tile_x in [x, x + size.x] {
        draw_texture_ex(
            texture,
            tile_x,
            y,
            TEXT_COLOR,
            DrawTextureParams {
                dest_size: Some(size),
                ..Default::default()
            },
        );
    }
}

fn draw_pipe(texture: &Texture2D, rect: Rect, is_top: bool) {
    let cap_height: f32 = (rect.w * PIPE_CAP_SOURCE.h / PIPE_CAP_SOURCE.w).min(rect.h);

    let body_height: f32 = rect.h - cap_height;

    let cap_y: f32 = if is_top { rect.y + body_height } else { rect.y };

    let body_y: f32 = if is_top { rect.y } else { rect.y + cap_height };

    for (source, y, height) in [
        (PIPE_BODY_SOURCE, body_y, body_height),
        (PIPE_CAP_SOURCE, cap_y, cap_height),
    ] {
        draw_texture_ex(
            texture,
            rect.x,
            y,
            TEXT_COLOR,
            DrawTextureParams {
                source: Some(source),
                dest_size: Some(vec2(rect.w, height)),
                flip_y: is_top,
                ..Default::default()
            },
        );
    }
}

fn draw_hero(texture: &Texture2D, hero: &Hero) {
    draw_texture_ex(
        texture,
        HERO_X - hero.size.x / 2.0,
        hero.y - hero.size.y / 2.0,
        TEXT_COLOR,
        DrawTextureParams {
            dest_size: Some(hero.size),
            rotation: hero.angle,
            pivot: Some(vec2(HERO_X, hero.y)),
            ..Default::default()
        },
    );
}

#[macroquad::main(window_conf)]
async fn main() {
    let background_texture: Texture2D = load_texture("assets/background.png")
        .await
        .expect("Failed to load assets/background.png");
    background_texture.set_filter(FilterMode::Linear);

    let background_scale: f32 = (screen_width() / background_texture.width())
        .max(screen_height() / background_texture.height());

    let background_size: Vec2 = vec2(
        background_texture.width() * background_scale,
        background_texture.height() * background_scale,
    );

    let pipe_texture: Texture2D = load_texture("assets/pipe.png")
        .await
        .expect("Failed to load assets/pipe.png");
    pipe_texture.set_filter(FilterMode::Linear);

    let hero_texture: Texture2D = load_texture("assets/hero.png")
        .await
        .expect("Failed to load assets/hero.png");
    hero_texture.set_filter(FilterMode::Linear);
    let hero_size: Vec2 = vec2(
        HERO_DRAW_WIDTH,
        HERO_DRAW_WIDTH * hero_texture.height() / hero_texture.width(),
    );

    let died_hero_texture: Texture2D = load_texture("assets/died-hero.png")
        .await
        .expect("Failed to load assets/died-hero.png");
    died_hero_texture.set_filter(FilterMode::Linear);

    let seed: u64 = (macroquad::miniquad::date::now() * 1_000_000.0) as u64;
    rand::srand(seed);

    let mut game: Game = Game::new(Hero {
        y: 360.0,
        velocity_y: 0.0,
        size: hero_size,
        angle: 0.0,
    });

    let mut danger_text_color: Color = WHITE;

    loop {
        let dt: f32 = get_frame_time();

        game.timer += dt;
        if game.timer >= game.timer_interval {
            game.timer -= game.timer_interval;

            danger_text_color = if danger_text_color == WHITE {
                RED
            } else {
                WHITE
            };
        }

        if is_key_pressed(KeyCode::R) {
            game.reset();
        }

        if game.state != GameState::GameOver && is_key_pressed(KeyCode::Space) {
            game.state = GameState::Playing;

            game.hero.velocity_y = FLAP_VELOCITY;
            game.hero.angle = HERO_UP_ANGLE;
        }

        if game.state == GameState::Playing {
            game.background_offset =
                (game.background_offset + BACKGROUND_SPEED * dt).rem_euclid(background_size.x);

            game.pipes
                .iter_mut()
                .for_each(|pipe| pipe.x -= game.pipe_speed * dt);

            game.hero.velocity_y += GRAVITY * dt;
            game.hero.y += game.hero.velocity_y * dt;

            let target_angle: f32 =
                (game.hero.velocity_y * 0.002).clamp(HERO_UP_ANGLE, HERO_DOWN_ANGLE);
            let blend: f32 = 1.0 - (-HERO_TILT_RESPONSE * dt).exp();
            game.hero.angle += (target_angle - game.hero.angle) * blend;

            let hero_g: Circle = Circle::new(HERO_X, game.hero.y, HERO_RADIUS);

            let touches_pipe: bool = game.pipes.iter().any(|pipe| {
                let (top_pipe, bottom_pipe) = pipe.rectangles();
                hero_g.overlaps_rect(&top_pipe) || hero_g.overlaps_rect(&bottom_pipe)
            });
            let touches_top: bool = hero_g.y - HERO_RADIUS <= 0.0;
            let touches_bottom: bool = hero_g.y + HERO_RADIUS >= screen_height();

            if touches_top || touches_bottom || touches_pipe {
                game.state = GameState::GameOver;

                game.hero.y = hero_g.y.clamp(HERO_RADIUS, screen_height() - HERO_RADIUS);
                game.hero.velocity_y = 0.0;
            }

            if game.state == GameState::Playing {
                game.pipes.iter_mut().for_each(|pipe| {
                    let passed: bool = pipe.x + PIPE_WIDTH < HERO_X - HERO_RADIUS;

                    if passed && !pipe.scored {
                        game.score += 1;
                        pipe.scored = true;

                        if game.score.is_multiple_of(10) {
                            game.gap_between_pipes -= 10.0;
                            game.pipe_speed += 10.0;
                        }
                    }
                });

                let mut rightmost_x = game
                    .pipes
                    .iter()
                    .map(|pipe| pipe.x)
                    .fold(f32::NEG_INFINITY, f32::max);

                game.pipes.iter_mut().for_each(|pipe| {
                    if pipe.x + PIPE_WIDTH < 0.0 {
                        rightmost_x += PIPE_SPACING;
                        *pipe = PipePair::new(rightmost_x, game.gap_between_pipes);
                    }
                });
            }
        }

        clear_background(BACKGROUND_COLOR);
        draw_background(&background_texture, background_size, game.background_offset);

        game.pipes.iter().for_each(|pipe| {
            let (top, bottom) = pipe.rectangles();

            draw_pipe(&pipe_texture, top, true);
            draw_pipe(&pipe_texture, bottom, false);
        });

        match game.state {
            GameState::Ready => {
                draw_text(
                    "SPACE - start",
                    20.0,
                    screen_height() - 20.0,
                    48.0,
                    TEXT_COLOR,
                );
            }
            GameState::Playing => {
                draw_hero(&hero_texture, &game.hero);
            }
            GameState::GameOver => {
                draw_hero(&died_hero_texture, &game.hero);

                draw_text(
                    "Game over! R - restart",
                    20.0,
                    screen_height() - 20.0,
                    32.0,
                    danger_text_color,
                );
            }
        };

        draw_text(
            format!("Score: {}", game.score),
            20.0,
            40.0,
            48.0,
            TEXT_COLOR,
        );

        next_frame().await;
    }
}

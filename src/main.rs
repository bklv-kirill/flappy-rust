use macroquad::prelude::*;

const GRAVITY: f32 = 1100.0;
const FLAP_VELOCITY: f32 = -350.0;

const CRAB_DRAW_WIDTH: f32 = 56.0;
const BACKGROUND_COLOR: Color = Color::from_rgba(24, 28, 40, 255);
const BACKGROUND_SPEED: f32 = 35.0;
const TEXT_COLOR: Color = WHITE;

const CRAB_RADIUS: f32 = 16.0;
const CRAB_X: f32 = 120.0;
const CRAB_UP_ANGLE: f32 = -0.35;
const CRAB_DOWN_ANGLE: f32 = 0.70;
const CRAB_TILE_RESPONSE: f32 = 12.0;

const PIPE_WIDTH: f32 = 80.0;
const PIPE_MARGIN: f32 = 100.0;
const PIPE_COUNT: usize = 3;
const PIPE_SPACING: f32 = 280.0;
const PIPE_SPEED: f32 = 200.0;
const PIPE_CAP_SOURCE: Rect = Rect::new(164.0, 0.0, 397.0, 170.0);
const PIPE_BODY_SOURCE: Rect = Rect::new(179.0, 180.0, 367.0, 1992.0);

#[derive(Clone, Copy, PartialEq)]
enum GameState {
    Ready,
    Playing,
    GameOver,
}

struct PipePair {
    x: f32,
    gap_y: f32,
    gap_between: f32,
    scored: bool,
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

fn create_pipes(gap_between: f32) -> Vec<PipePair> {
    (0..PIPE_COUNT)
        .map(|i| {
            let x: f32 = screen_width() + (i as f32) * PIPE_SPACING;

            PipePair::new(x, gap_between)
        })
        .collect()
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

#[macroquad::main(window_conf)]
async fn main() {
    let background_texture: Texture2D = load_texture("assets/background.png")
        .await
        .expect("Failed to load assets/background.png");
    background_texture.set_filter(FilterMode::Linear);

    let bacground_scale: f32 = (screen_width() / background_texture.width())
        .max(screen_height() / background_texture.height());

    let background_size: Vec2 = vec2(
        background_texture.width() * bacground_scale,
        background_texture.height() * bacground_scale,
    );
    let mut background_offset: f32 = 0.0;

    let pipe_texture: Texture2D = load_texture("assets/pipe.png")
        .await
        .expect("Failed to load assets/pipe.png");
    pipe_texture.set_filter(FilterMode::Linear);

    let crab_texture: Texture2D = load_texture("assets/ferris-cyber.png")
        .await
        .expect("Failed to load assets/ferris-cyber.png");
    crab_texture.set_filter(FilterMode::Linear);
    let crab_size: Vec2 = vec2(
        CRAB_DRAW_WIDTH,
        CRAB_DRAW_WIDTH * crab_texture.height() / crab_texture.width(),
    );

    let seed: u64 = (macroquad::miniquad::date::now() * 1_000_000.0) as u64;
    rand::srand(seed);

    let mut state: GameState = GameState::Ready;
    let mut score: u32 = 0;

    let mut crab_y: f32 = 360.0;
    let mut crab_velocity_y: f32 = 0.0;
    let mut crab_angle: f32 = 0.0;

    let mut gap_between_pipes: f32 = 200.0;
    let mut pipes: Vec<PipePair> = create_pipes(gap_between_pipes);
    let mut pipe_speed: f32 = 200.0;

    loop {
        let dt: f32 = get_frame_time();

        if is_key_pressed(KeyCode::R) {
            state = GameState::Ready;
            score = 0;

            crab_y = 360.0;
            crab_velocity_y = 0.0;
            crab_angle = 0.0;

            gap_between_pipes = PIPE_SPACING;
            pipes = create_pipes(gap_between_pipes);
            pipe_speed = PIPE_SPEED;

            background_offset = 0.0;
        }

        if state != GameState::GameOver && is_key_pressed(KeyCode::Space) {
            state = GameState::Playing;

            crab_velocity_y = FLAP_VELOCITY;
            crab_angle = CRAB_UP_ANGLE;
        }

        if state == GameState::Playing {
            background_offset =
                (background_offset + BACKGROUND_SPEED * dt).rem_euclid(background_size.x);

            pipes.iter_mut().for_each(|pipe| pipe.x -= pipe_speed * dt);

            crab_velocity_y += GRAVITY * dt;
            crab_y += crab_velocity_y * dt;

            let target_angle: f32 = (crab_velocity_y * 0.002).clamp(CRAB_UP_ANGLE, CRAB_DOWN_ANGLE);
            let blend: f32 = 1.0 - (-CRAB_TILE_RESPONSE * dt).exp();
            crab_angle += (target_angle - crab_angle) * blend;

            let crab: Circle = Circle::new(CRAB_X, crab_y, CRAB_RADIUS);

            let touches_pipe: bool = pipes.iter().any(|pipe| {
                let (top_pipe, bottom_pipe) = pipe.rectangles();
                crab.overlaps_rect(&top_pipe) || crab.overlaps_rect(&bottom_pipe)
            });
            let touches_top: bool = crab_y - CRAB_RADIUS <= 0.0;
            let touches_bottom: bool = crab_y + CRAB_RADIUS >= screen_height();

            if touches_top || touches_bottom || touches_pipe {
                state = GameState::GameOver;

                crab_y = crab_y.clamp(CRAB_RADIUS, screen_height() - CRAB_RADIUS);
                crab_velocity_y = 0.0;
            }

            if state == GameState::Playing {
                pipes.iter_mut().for_each(|pipe| {
                    let passed: bool = pipe.x + PIPE_WIDTH < CRAB_X - CRAB_RADIUS;

                    if passed && !pipe.scored {
                        score += 1;
                        pipe.scored = true;

                        if score.is_multiple_of(10) {
                            gap_between_pipes -= 10.0;
                            pipe_speed += 10.0;
                        }
                    }
                });

                let mut rightmost_x = pipes
                    .iter()
                    .map(|pipe| pipe.x)
                    .fold(f32::NEG_INFINITY, f32::max);

                pipes.iter_mut().for_each(|pipe| {
                    if pipe.x + PIPE_WIDTH < 0.0 {
                        rightmost_x += PIPE_SPACING;
                        *pipe = PipePair::new(rightmost_x, gap_between_pipes);
                    }
                });
            }
        }

        clear_background(BACKGROUND_COLOR);
        draw_background(&background_texture, background_size, background_offset);

        pipes.iter().for_each(|pipe| {
            let (top, bottom) = pipe.rectangles();

            draw_pipe(&pipe_texture, top, true);
            draw_pipe(&pipe_texture, bottom, false);
        });

        draw_texture_ex(
            &crab_texture,
            CRAB_X - crab_size.x / 2.0,
            crab_y - crab_size.y / 2.0,
            TEXT_COLOR,
            DrawTextureParams {
                dest_size: Some(crab_size),
                rotation: crab_angle,
                pivot: Some(vec2(CRAB_X, crab_y)),
                ..Default::default()
            },
        );

        draw_text(&format!("Score: {score}"), 20.0, 40.0, 48.0, TEXT_COLOR);

        match state {
            GameState::Ready => {
                draw_text("SPACE - start", 130.0, 320.0, 48.0, TEXT_COLOR);
            }
            GameState::Playing => {}
            GameState::GameOver => {
                draw_text("Game over! R - restart", 70.0, 320.0, 32.0, TEXT_COLOR);
            }
        }

        next_frame().await;
    }
}

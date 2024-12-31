use core::panic;
use std::time::Duration;
use std::time::Instant;

use ggez::conf::WindowMode;
use ggez::event;
use ggez::glam::*;
use ggez::graphics::Drawable;
use ggez::graphics::FillOptions;
use ggez::graphics::PxScale;
use ggez::graphics::Rect;
use ggez::graphics::StrokeOptions;
use ggez::graphics::TextFragment;
use ggez::graphics::{self, Color};
use ggez::{Context, GameResult};
use turing_lib::machine::Symbol;
use turing_lib::machine::TapeSide;
use turing_lib::machine::TickResult;
use turing_lib::machine::TuringMachine;
use turing_lib::tape::Tape;

const WINDOW_WIDTH: f32 = 1000.0;
const WINDOW_HEIGHT: f32 = 800.0;

const HORIZ_MARGIN: f32 = 80.0;
const CELL_COUNT: f32 = 7.0;
const CELL_SIZE: f32 = (WINDOW_WIDTH - HORIZ_MARGIN * 2.0) / CELL_COUNT;

const HEAD_TRIANGLE_WIDTH: f32 = CELL_SIZE / 3.0;
const HEAD_TRIANGLE_HEIGHT: f32 = CELL_SIZE / 2.4;
const HEAD_TRIANGLE_MARGIN: f32 = CELL_SIZE / 8.0;

const WRITE_ANIM_MAX_ALPHA: f32 = 0.8;

struct AnimationState {
    animation: Animation,
    stage_begin: Instant,
    next_stage: Instant,
}

enum Animation {
    FirstWait,
    HeadMove {
        delta: f32, // -1, 0 or 1, depending on where the head is moving (0 if not).
        current_text_displacement: f32, // 0.0 to 1.0 percent on the current text displacement.
    },
    LastWait,
}

struct MainState {
    turing_machine: TuringMachine,

    writing_animation: Option<f32>, // Where f32 is the alpha value [0.0, WRITE_ANIM_MAX_ALPHA]

    visual_tape: Tape,
    visual_head_idx: usize,

    should_update: bool,
    animation_state: Option<AnimationState>,
    // text_displacement_percent: f32,
    // anim_delta: f32, // -1 -> To the left, 0 -> Stay, 1 -> To the right
    last_tick: Option<TickResult>,
}

impl MainState {
    fn new() -> GameResult<MainState> {
        let mut s = MainState {
            turing_machine: TuringMachine::new_from_file("main.tng", "1010").unwrap(),

            writing_animation: None,

            last_tick: None,

            visual_tape: Tape::new(vec![]),
            visual_head_idx: 0,
            animation_state: Some(AnimationState {
                animation: Animation::LastWait,
                stage_begin: Instant::now(),
                next_stage: Instant::now() + Duration::from_millis(LAST_WAIT_DURATION_MS),
            }),
            should_update: true,
            // text_displacement_percent: 0.0,
            // anim_delta: 0.0,
        };

        s.visual_head_idx = s.turing_machine.head_idx();
        s.visual_tape = s.turing_machine.tape().clone();

        Ok(s)
    }
}

const FIRST_WAIT_DURATION_MS: u64 = 300;
const HEAD_MOVE_DURATION_MS: u64 = 1000;
const LAST_WAIT_DURATION_MS: u64 = 300;

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if self.turing_machine.is_halted() {
            return Ok(());
        }

        if let Some(ref mut animation_state) = self.animation_state {
            if Instant::now() >= animation_state.next_stage {
                let (new_animation, animation_duration) = match animation_state.animation {
                    Animation::FirstWait => {
                        self.writing_animation = None;

                        let anim_delta = if let Some(last_tick) = &self.last_tick {
                            if let Some(TapeSide::Left) = last_tick.extended_tape_on_side {
                                -1.0
                            } else {
                                self.turing_machine.head_idx() as f32 - self.visual_head_idx as f32
                            }
                        } else {
                            0.0
                        };
                        (
                            Animation::HeadMove {
                                delta: anim_delta,
                                current_text_displacement: 0.0,
                            },
                            Duration::from_millis(HEAD_MOVE_DURATION_MS),
                        )
                    }
                    Animation::HeadMove { .. } => {
                        self.visual_head_idx = self.turing_machine.head_idx();
                        self.should_update = true;
                        (
                            Animation::LastWait,
                            Duration::from_millis(LAST_WAIT_DURATION_MS),
                        )
                    }
                    Animation::LastWait => {
                        self.visual_tape = self.turing_machine.tape().clone();
                        (
                            Animation::FirstWait,
                            Duration::from_millis(FIRST_WAIT_DURATION_MS),
                        )
                    }
                };

                *animation_state = AnimationState {
                    animation: new_animation,
                    stage_begin: Instant::now(),
                    next_stage: Instant::now() + animation_duration,
                };
            }
        }

        if let Some(ref mut animation_state) = &mut self.animation_state {
            let total_duration = animation_state.next_stage - animation_state.stage_begin;
            let duration_since_begin = Instant::now() - animation_state.stage_begin;

            let percent = duration_since_begin.as_millis() * 100 / total_duration.as_millis();

            if let Animation::HeadMove {
                delta,
                ref mut current_text_displacement,
            } = &mut animation_state.animation
            {
                *current_text_displacement = *delta * percent as f32 / 100.0;
            } else if let Some(ref mut alpha) = self.writing_animation {
                let percent = (percent * 2).min(100); // Speed up opacity transition by 2

                let new_alpha = percent as f32 * WRITE_ANIM_MAX_ALPHA / 100.0;

                if let Animation::LastWait = animation_state.animation {
                    *alpha = new_alpha;
                } else {
                    *alpha = 1.0 - new_alpha;
                }
            }
        }

        // Update machine
        if !self.should_update {
            return Ok(());
        }

        let mut prev_tape_content = self.turing_machine.tape().get_content().to_vec();
        // let mut prev_tape_content = self.turing_machine.tape().0.clone();
        let tick_result = self.turing_machine.tick();

        if let Some(TapeSide::Left) = tick_result.extended_tape_on_side {
            self.visual_head_idx += 1;
            prev_tape_content.insert(0, Symbol::Blank);
            self.visual_tape = Tape::new(prev_tape_content);
        }

        if tick_result.written_different_symbol {
            self.writing_animation = Some(0.0);
        } else {
            self.writing_animation = None;
        }
        self.should_update = false;
        self.last_tick = Some(tick_result);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let bg_color = graphics::Color::from([0.1, 0.1, 0.1, 1.0]);
        let head_color = Color::RED;
        let mut canvas = graphics::Canvas::from_frame(ctx, bg_color);

        let stroke_width = (CELL_SIZE / 2.0 * 0.03).ceil().max(1.0);
        let head_stroke_width = (CELL_SIZE / 2.0 * 0.07).ceil().max(1.0);

        let horiz_line = graphics::Mesh::new_line(
            ctx,
            &[
                [HORIZ_MARGIN - stroke_width / 2.0 - CELL_SIZE, 0.0],
                [
                    WINDOW_WIDTH - HORIZ_MARGIN + stroke_width / 2.0 + CELL_SIZE,
                    0.0,
                ],
            ],
            stroke_width,
            Color::WHITE,
        )?;
        canvas.draw(&horiz_line, [0.0, WINDOW_HEIGHT / 2.0 - CELL_SIZE / 2.0]);
        canvas.draw(&horiz_line, [0.0, WINDOW_HEIGHT / 2.0 + CELL_SIZE / 2.0]);

        let mut text_displacement_percent = 0.0;
        if let Some(animation_state) = &self.animation_state {
            if let Animation::HeadMove {
                current_text_displacement,
                ..
            } = animation_state.animation
            {
                text_displacement_percent = current_text_displacement;
            }
        }

        let vert_line = graphics::Mesh::new_line(
            ctx,
            &[[0.0, 0.0], [0.0, CELL_SIZE]],
            stroke_width,
            Color::WHITE,
        )?;
        for i in 0..=(CELL_COUNT as usize + 1) {
            canvas.draw(
                &vert_line,
                [
                    HORIZ_MARGIN + CELL_SIZE * (i as f32) - CELL_SIZE * text_displacement_percent,
                    WINDOW_HEIGHT / 2.0 - CELL_SIZE / 2.0,
                ],
            );
        }

        let head_triangle = graphics::Mesh::new_polygon(
            ctx,
            graphics::DrawMode::Fill(FillOptions::default()),
            &[
                [HEAD_TRIANGLE_WIDTH / 2.0, 0.0],
                [0.0, HEAD_TRIANGLE_HEIGHT],
                [HEAD_TRIANGLE_WIDTH, HEAD_TRIANGLE_HEIGHT],
            ],
            head_color,
        )?;
        canvas.draw(
            &head_triangle,
            [
                WINDOW_WIDTH / 2.0 - HEAD_TRIANGLE_WIDTH / 2.0,
                WINDOW_HEIGHT / 2.0 + CELL_SIZE / 2.0 + HEAD_TRIANGLE_MARGIN,
            ],
        );

        // + 1 to also draw non visible border cells
        for i in -(CELL_COUNT as isize / 2 + 1)..=(CELL_COUNT as isize / 2 + 1) {
            let text_fragment = {
                let correct_index = self.visual_head_idx as isize + i;

                let char_at = {
                    if correct_index < 0 || correct_index >= self.visual_tape.len() as isize {
                        self.turing_machine.blank_symbol()
                    } else {
                        match self.visual_tape.read(correct_index as usize) {
                            Symbol::Blank => self.turing_machine.blank_symbol(),
                            Symbol::Mark(c) => c,
                            _ => unreachable!("Default Symbol won't be present in the tape."),
                        }
                    }
                };
                let text_content: String = format!("{char_at}");
                let text_size = CELL_SIZE * 0.75;

                let mut text_fragment = TextFragment::default();
                text_fragment.text = text_content;
                // text_fragment.color = Some(Color::new(1.0, 1.0, 1.0, 0.0));
                text_fragment.scale(PxScale {
                    x: text_size,
                    y: text_size,
                })
            };
            let text_piece = graphics::Text::new(text_fragment);
            let Rect {
                w: text_width,
                h: text_height,
                ..
            } = text_piece.dimensions(ctx).unwrap();

            canvas.draw(
                &text_piece,
                [
                    (CELL_SIZE * (i as f32) + WINDOW_WIDTH / 2.0)
                        - text_width / 2.0
                        - CELL_SIZE * text_displacement_percent,
                    WINDOW_HEIGHT / 2.0 - text_height / 2.0,
                ],
            );

            if i == 0 {
                if let Some(alpha) = self.writing_animation {
                    let write_opacity_square = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::Fill(FillOptions::default()),
                        Rect::new(0.0, 0.0, CELL_SIZE, CELL_SIZE),
                        Color::new(bg_color.r, bg_color.b, bg_color.g, alpha),
                    )?;

                    canvas.draw(
                        &write_opacity_square,
                        [
                            (CELL_SIZE * (i as f32) + WINDOW_WIDTH / 2.0) - CELL_SIZE / 2.0,
                            WINDOW_HEIGHT / 2.0 - CELL_SIZE / 2.0,
                        ],
                    );
                }
            }
        }

        // Draw hidden border squares
        let square = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::Fill(FillOptions::default()),
            Rect::new(0.0, 0.0, HORIZ_MARGIN, CELL_SIZE + 10.0),
            bg_color,
        )?;
        canvas.draw(
            &square,
            [0.0, WINDOW_HEIGHT / 2.0 - (CELL_SIZE + 10.0) / 2.0],
        );
        canvas.draw(
            &square,
            [
                WINDOW_WIDTH - HORIZ_MARGIN,
                WINDOW_HEIGHT / 2.0 - (CELL_SIZE + 10.0) / 2.0,
            ],
        );

        let head_square = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::Stroke(StrokeOptions::default().with_line_width(head_stroke_width)),
            Rect::new(0.0, 0.0, CELL_SIZE, CELL_SIZE),
            head_color,
        )?;
        canvas.draw(
            &head_square,
            [
                WINDOW_WIDTH / 2.0 - CELL_SIZE / 2.0,
                WINDOW_HEIGHT / 2.0 - CELL_SIZE / 2.0,
            ],
        );

        if self.turing_machine.is_halted() {
            let (text_content, text_color) = if self.turing_machine.is_accepting() {
                ("Halted, accepts", Color::GREEN)
            } else {
                ("Halted, rejects", Color::RED)
            };

            self.animation_state = None;
            //self.animation_percent = 0.0;
            let horiz_text_margin = 20.0;
            let vert_text_margin = 50.0;
            let text_fragment = {
                let text_size = 30.0;

                let mut text_fragment = TextFragment::default();
                text_fragment.text = text_content.to_string();
                text_fragment.color = Some(text_color);
                text_fragment.scale(PxScale {
                    x: text_size,
                    y: text_size,
                })
            };
            let text_piece = graphics::Text::new(text_fragment);
            let Rect {
                w: text_width,
                h: _text_height,
                ..
            } = text_piece.dimensions(ctx).unwrap();

            canvas.draw(
                &text_piece,
                [
                    WINDOW_WIDTH - horiz_text_margin - text_width,
                    vert_text_margin,
                ],
            );
        }

        {
            let text_margins = 20.0;
            let text_fragment = {
                let text_content = format!(
                    "Current state: \"{}\"",
                    self.turing_machine.current_state_name()
                );
                let text_size = 20.0;

                let mut text_fragment = TextFragment::default();
                text_fragment.text = text_content.to_string();
                text_fragment.scale(PxScale {
                    x: text_size,
                    y: text_size,
                })
            };
            let text_piece = graphics::Text::new(text_fragment);
            let Rect {
                w: text_width,
                h: _text_height,
                ..
            } = text_piece.dimensions(ctx).unwrap();

            canvas.draw(
                &text_piece,
                [WINDOW_WIDTH - text_margins - text_width, text_margins],
            );
        }

        canvas.finish(ctx)?;
        Ok(())
    }
}

pub fn main() -> GameResult {
    if CELL_COUNT as isize % 2 == 0 {
        panic!("Cell count must be an odd positive whole float.");
    }

    let cb = ggez::ContextBuilder::new("Turing Machine Simulator", "keelus");
    let (ctx, event_loop) = cb
        .window_mode({
            let mut window_mode = WindowMode::default();
            window_mode.width = WINDOW_WIDTH;
            window_mode.height = WINDOW_HEIGHT;
            window_mode
        })
        .build()?;
    let state = MainState::new()?;
    event::run(ctx, event_loop, state)
}

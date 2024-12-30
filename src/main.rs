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
use turing_lib::machine::TuringMachine;
use turing_lib::tape::Tape;

const TARGET_UPDATES_PER_SECOND: usize = 1;

const WINDOW_WIDTH: f32 = 1000.0;
const WINDOW_HEIGHT: f32 = 800.0;

const HORIZ_MARGIN: f32 = 80.0;
const CELL_COUNT: f32 = 7.0;
const CELL_SIZE: f32 = (WINDOW_WIDTH - HORIZ_MARGIN * 2.0) / CELL_COUNT;

const HEAD_TRIANGLE_WIDTH: f32 = 40.0;
const HEAD_TRIANGLE_HEIGHT: f32 = 50.0;

const WRITE_ANIM_MAX_ALPHA: f32 = 0.8;

struct AnimationState {
    animation: Animation,
    stage_begin: Instant,
    next_stage: Instant,
}

enum Animation {
    FirstWait,
    HeadMove,
    LastWait,
}

struct MainState {
    turing_machine: TuringMachine,
    last_update: Option<Instant>,

    doing_write_anim: bool,
    written_cell_opacity: f32,

    current_updated: bool,

    visual_tape: Tape,
    head_idx: usize,

    animation_state: Option<AnimationState>,
    text_displacement_percent: f32,
    anim_delta: f32, // -1 -> To the left, 0 -> Stay, 1 -> To the right
}

impl MainState {
    fn new() -> GameResult<MainState> {
        let mut s = MainState {
            turing_machine: TuringMachine::new_from_file("main.tng", "1111").unwrap(),
            last_update: None,

            written_cell_opacity: 0.0,
            doing_write_anim: false,

            visual_tape: Tape::new(vec![]),
            current_updated: true,
            head_idx: 0,
            animation_state: Some(AnimationState {
                animation: Animation::LastWait,
                stage_begin: Instant::now(),
                next_stage: Instant::now() + Duration::from_millis(LAST_WAIT_DURATION_MS),
            }),
            text_displacement_percent: 0.0,
            anim_delta: 0.0,
        };

        s.head_idx = s.turing_machine.head_idx;
        s.visual_tape = Tape::new(s.turing_machine.tape.0.clone());

        Ok(s)
    }
}

const FIRST_WAIT_DURATION_MS: u64 = 300;
const HEAD_MOVE_DURATION_MS: u64 = 1000;
const LAST_WAIT_DURATION_MS: u64 = 300;

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let bg_color = graphics::Color::from([0.1, 0.1, 0.1, 1.0]);

        // Update animation
        // let fps = timer::fps(ctx);
        // let animation_step = 4.0 / fps;
        let mut should_update = false;
        if let Some(ref mut animation_state) = self.animation_state {
            if Instant::now() >= animation_state.next_stage {
                // Transition to next animation
                let (new_animation, animation_duration) = match animation_state.animation {
                    Animation::FirstWait => {
                        self.written_cell_opacity = 0.0;
                        self.doing_write_anim = false;
                        (
                            Animation::HeadMove,
                            Duration::from_millis(HEAD_MOVE_DURATION_MS),
                        )
                    }
                    Animation::HeadMove => {
                        self.text_displacement_percent = 0.0;
                        self.head_idx = self.turing_machine.head_idx;
                        should_update = true;
                        (
                            Animation::LastWait,
                            Duration::from_millis(LAST_WAIT_DURATION_MS),
                        )
                    }
                    Animation::LastWait => {
                        self.text_displacement_percent = 0.0;
                        // self.written_cell_opacity = 0.0;
                        self.visual_tape = Tape::new(self.turing_machine.tape.0.clone());
                        // self.current_updated = false;
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
        } else {
            should_update = true;
        }

        if let Some(animation_state) = &self.animation_state {
            if let Animation::HeadMove = animation_state.animation {
                let total_duration = animation_state.next_stage - animation_state.stage_begin;
                let duration_since_begin = Instant::now() - animation_state.stage_begin;

                let percent = duration_since_begin.as_millis() * 100 / total_duration.as_millis();
                self.text_displacement_percent = self.anim_delta * percent as f32 / 100.0;
            } else if let Animation::LastWait = animation_state.animation {
                if self.doing_write_anim {
                    let total_duration = animation_state.next_stage - animation_state.stage_begin;
                    let duration_since_begin = Instant::now() - animation_state.stage_begin;

                    let percent =
                        duration_since_begin.as_millis() * 100 / total_duration.as_millis() * 2;
                    let percent = percent.min(100);

                    let alpha = percent as f32 * WRITE_ANIM_MAX_ALPHA / 100.0;
                    self.written_cell_opacity = alpha;
                }
            } else {
                if self.doing_write_anim {
                    let total_duration = animation_state.next_stage - animation_state.stage_begin;
                    let duration_since_begin = Instant::now() - animation_state.stage_begin;

                    let percent =
                        duration_since_begin.as_millis() * 100 / total_duration.as_millis() * 2;
                    let percent = percent.min(100);

                    let alpha = percent as f32 * WRITE_ANIM_MAX_ALPHA / 100.0;
                    self.written_cell_opacity = 1.0 - alpha;
                }
            }
        }

        // Update machine
        if !should_update || self.turing_machine.halted {
            return Ok(());
        }
        if let Some(last_update) = self.last_update {
            let target_duration_millis = 1000 / TARGET_UPDATES_PER_SECOND as u128;
            if Instant::now().duration_since(last_update).as_millis() < target_duration_millis {
                return Ok(());
            }
        }

        self.last_update = Some(Instant::now());
        self.head_idx = self.turing_machine.head_idx;

        let len_before = self.turing_machine.tape.len();
        let tape_before = self.turing_machine.tape.0.clone();
        println!("Tick");
        print!("\"{}\"", self.turing_machine.tape);
        self.turing_machine.tick();
        let tape_after = self.turing_machine.tape.0.clone();
        self.anim_delta =
            if self.turing_machine.tape.len() > len_before && self.turing_machine.head_idx == 0 {
                // Tape extended to the left
                self.head_idx += 1;
                -1.0
            } else {
                self.turing_machine.head_idx as f32 - self.head_idx as f32
            };

        println!(" vs \"{}\"", self.turing_machine.tape);

        // Tape updated only is true when the tape is different and it isn't because
        // of extending the tape in the left or right.
        // TODO: Fix writing and extending
        let tape_updated = tape_before != tape_after
            && !((self.turing_machine.tape.len() > len_before
                && self.turing_machine.head_idx == 0)
                || (self.turing_machine.tape.len() > len_before
                    && self.turing_machine.head_idx == self.turing_machine.tape.len() - 1));

        if tape_updated {
            println!("Tape updated!");
            self.doing_write_anim = true;
        } else {
            self.written_cell_opacity = 0.0;
            self.doing_write_anim = false;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let bg_color = graphics::Color::from([0.1, 0.1, 0.1, 1.0]);
        let head_color = graphics::Color::from([1.0, 0.0, 0.0, 1.0]);
        let mut canvas = graphics::Canvas::from_frame(ctx, bg_color);

        let stroke_width = 2.0;
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
                    HORIZ_MARGIN + CELL_SIZE * (i as f32)
                        - CELL_SIZE * self.text_displacement_percent,
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
                WINDOW_HEIGHT / 2.0 + CELL_SIZE / 2.0 + 20.0,
            ],
        );

        // + 1 to also draw non visible border cells
        for i in -(CELL_COUNT as isize / 2 + 1)..=(CELL_COUNT as isize / 2 + 1) {
            let text_fragment = {
                // let diff = self.turing_machine.head_idx - self.head_idx;
                // println!("Diff: {diff}");
                // let correct_index = self.head_idx as isize + i + diff as isize;
                let correct_index = self.head_idx as isize + i;

                let char_at = {
                    if correct_index < 0 || correct_index >= self.visual_tape.len() as isize {
                        self.turing_machine.blank_symbol
                    } else {
                        match self.visual_tape.read(correct_index as usize) {
                            Symbol::Blank => self.turing_machine.blank_symbol,
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
                        - CELL_SIZE * self.text_displacement_percent,
                    WINDOW_HEIGHT / 2.0 - text_height / 2.0,
                ],
            );

            if i == 0 {
                let write_opacity_square = graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::Fill(FillOptions::default()),
                    Rect::new(0.0, 0.0, CELL_SIZE, CELL_SIZE),
                    Color::new(
                        bg_color.r,
                        bg_color.b,
                        bg_color.g,
                        self.written_cell_opacity,
                    ),
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

        // Draw hidden border squares
        let square = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::Fill(FillOptions::default()),
            Rect::new(0.0, 0.0, CELL_SIZE, CELL_SIZE + 10.0),
            bg_color,
        )?;
        canvas.draw(
            &square,
            [
                (HORIZ_MARGIN - CELL_SIZE),
                WINDOW_HEIGHT / 2.0 - (CELL_SIZE + 10.0) / 2.0,
            ],
        );
        canvas.draw(
            &square,
            [
                WINDOW_WIDTH - (HORIZ_MARGIN - CELL_SIZE) - CELL_SIZE,
                WINDOW_HEIGHT / 2.0 - (CELL_SIZE + 10.0) / 2.0,
            ],
        );

        let head_square = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::Stroke(StrokeOptions::default().with_line_width(4.0)),
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

        if self.turing_machine.halted {
            self.animation_state = None;
            //self.animation_percent = 0.0;
            let horiz_text_margin = 20.0;
            let vert_text_margin = 50.0;
            let text_fragment = {
                let text_content = "Halted";
                let text_size = 30.0;

                let mut text_fragment = TextFragment::default();
                text_fragment.text = text_content.to_string();
                text_fragment.color = Some(Color::RED);
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
                let text_content =
                    format!("Current state: \"{}\"", self.turing_machine.current_state);
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

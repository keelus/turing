use ggez::{
    event::{self, MouseButton},
    glam::*,
    graphics::{self, Color, Drawable, FillOptions, PxScale, Rect, StrokeOptions, TextFragment},
    input::mouse::{set_cursor_type, CursorIcon},
    mint::Point2,
    Context, GameError, GameResult,
};
use num_input::NumberInput;
use std::{
    env::{self, args},
    path,
    process::exit,
    time::{Duration, Instant},
};
use turing_lib::{
    machine::{Symbol, TickResult, TuringMachine},
    tape::{Tape, TapeSide},
};

mod num_input;

const HORIZ_MARGIN: f32 = 80.0;

const DEFAULT_CELL_COUNT: usize = 7;
const WRITE_ANIM_MAX_ALPHA: f32 = 0.8;

const FIRST_WAIT_DURATION_MS: u64 = 100;
const HEAD_MOVE_DURATION_MS: u64 = 333;
const LAST_WAIT_DURATION_MS: u64 = 100;

const ACCENT_COLOR: Color = Color {
    r: 110.0 / 255.0,
    g: 157.0 / 255.0,
    b: 209.0 / 255.0,
    a: 1.0,
};

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

struct Sizing {
    window: Point2<f32>,

    cell_size: f32,

    head_triangle: Point2<f32>,
    head_triangle_margin: f32,
}

impl Sizing {
    pub fn calculate(window_width: f32, window_height: f32, cell_count: usize) -> Self {
        const HORIZ_MARGIN: f32 = 80.0;
        let cell_size = (window_width - HORIZ_MARGIN * 2.0) / cell_count as f32;
        Self {
            window: [window_width, window_height].into(),

            cell_size,

            head_triangle: [cell_size / 3.0, cell_size / 2.4].into(),
            head_triangle_margin: cell_size / 8.0,
        }
    }
}

struct MainState {
    turing_machine: TuringMachine,

    writing_animation: Option<f32>, // Where f32 is the alpha value [0.0, WRITE_ANIM_MAX_ALPHA]

    visual_tape: Tape,
    visual_head_idx: usize,

    should_update: bool,
    animation_state: Option<AnimationState>,
    last_tick: Option<TickResult>,

    speed_input: NumberInput,
    cells_input: NumberInput,

    sizing: Sizing,
    light_theme: bool,
}

impl MainState {
    fn new(
        filename: &str,
        tape: &str,
        window_width: f32,
        window_height: f32,
        light_theme: bool,
    ) -> GameResult<MainState> {
        let mut s = MainState {
            turing_machine: TuringMachine::new_from_file(filename, tape)
                .map_err(|err| GameError::CustomError(err))?,

            writing_animation: None,

            last_tick: None,

            visual_tape: Tape::new(vec![]),
            visual_head_idx: 0,
            animation_state: Some(AnimationState {
                animation: Animation::LastWait,
                stage_begin: Instant::now(),
                next_stage: Instant::now() + Duration::from_millis(1000),
            }),
            should_update: true,
            sizing: Sizing::calculate(window_width, window_height, DEFAULT_CELL_COUNT),

            cells_input: NumberInput::new(
                "Visible cells",
                7,
                2,
                (3, 71),
                Rect::new(30.0, window_height - 120.0, 100.0, 30.0),
                if light_theme {
                    Color::BLACK
                } else {
                    Color::WHITE
                },
            ),
            speed_input: NumberInput::new(
                "Simulation speed",
                3,
                1,
                (1, 5),
                Rect::new(30.0, window_height - 50.0, 100.0, 30.0),
                if light_theme {
                    Color::BLACK
                } else {
                    Color::WHITE
                },
            ),
            light_theme,
        };

        s.visual_head_idx = s.turing_machine.head_idx();
        s.visual_tape = s.turing_machine.tape().clone();

        Ok(s)
    }

    pub fn get_colors(&self) -> (Color, Color) {
        let bg_color = if self.light_theme {
            Color::WHITE
        } else {
            Color::from_rgb(22, 23, 25)
        };
        let fg_color = if self.light_theme {
            Color::from_rgb(68, 68, 68)
        } else {
            Color::from_rgb(224, 224, 224)
        };
        (bg_color, fg_color)
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if self.turing_machine.is_halted() {
            return Ok(());
        }

        if let Some(ref mut animation_state) = self.animation_state {
            if Instant::now() >= animation_state.next_stage {
                let speed_multiplier = (1.0 - self.speed_input.percent()) * 4.0 + 1.0;
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
                            Duration::from_millis(
                                (HEAD_MOVE_DURATION_MS as f32 * speed_multiplier) as u64,
                            ),
                        )
                    }
                    Animation::HeadMove { .. } => {
                        self.visual_head_idx = self.turing_machine.head_idx();
                        self.should_update = true;
                        (
                            Animation::LastWait,
                            Duration::from_millis(
                                (LAST_WAIT_DURATION_MS as f32 * speed_multiplier) as u64,
                            ),
                        )
                    }
                    Animation::LastWait => {
                        self.visual_tape = self.turing_machine.tape().clone();
                        (
                            Animation::FirstWait,
                            Duration::from_millis(
                                (FIRST_WAIT_DURATION_MS as f32 * speed_multiplier) as u64,
                            ),
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
        let (bg_color, fg_color) = self.get_colors();

        let mut canvas = graphics::Canvas::from_frame(ctx, bg_color);

        let stroke_width = (self.sizing.cell_size / 2.0 * 0.03).ceil().max(1.0);
        let head_stroke_width = (self.sizing.cell_size / 2.0 * 0.07).ceil().max(1.0);

        let horiz_line = graphics::Mesh::new_line(
            ctx,
            &[
                [
                    HORIZ_MARGIN - stroke_width / 2.0 - self.sizing.cell_size,
                    0.0,
                ],
                [
                    self.sizing.window.x - HORIZ_MARGIN
                        + stroke_width / 2.0
                        + self.sizing.cell_size,
                    0.0,
                ],
            ],
            stroke_width,
            fg_color,
        )?;
        canvas.draw(
            &horiz_line,
            [
                0.0,
                self.sizing.window.y / 2.0 - self.sizing.cell_size / 2.0,
            ],
        );
        canvas.draw(
            &horiz_line,
            [
                0.0,
                self.sizing.window.y / 2.0 + self.sizing.cell_size / 2.0,
            ],
        );

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
            &[[0.0, 0.0], [0.0, self.sizing.cell_size]],
            stroke_width,
            fg_color,
        )?;
        for i in 0..=(self.cells_input.value() as usize + 1) {
            canvas.draw(
                &vert_line,
                [
                    HORIZ_MARGIN + self.sizing.cell_size * (i as f32)
                        - self.sizing.cell_size * text_displacement_percent,
                    self.sizing.window.y / 2.0 - self.sizing.cell_size / 2.0,
                ],
            );
        }

        let head_triangle = graphics::Mesh::new_polygon(
            ctx,
            graphics::DrawMode::Fill(FillOptions::default()),
            &[
                [self.sizing.head_triangle.x / 2.0, 0.0],
                [0.0, self.sizing.head_triangle.y],
                [self.sizing.head_triangle.x, self.sizing.head_triangle.y],
            ],
            ACCENT_COLOR,
        )?;
        canvas.draw(
            &head_triangle,
            [
                self.sizing.window.x / 2.0 - self.sizing.head_triangle.x / 2.0,
                self.sizing.window.y / 2.0
                    + self.sizing.cell_size / 2.0
                    + self.sizing.head_triangle_margin,
            ],
        );

        // + 1 to also draw non visible border cells
        for i in -(self.cells_input.value() as isize / 2 + 1)
            ..=(self.cells_input.value() as isize / 2 + 1)
        {
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
            let text_size = self.sizing.cell_size * 0.75;

            let text_fragment = TextFragment {
                text: text_content,
                font: None,
                scale: Some(PxScale {
                    x: text_size,
                    y: text_size,
                }),
                color: Some(fg_color),
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
                    (self.sizing.cell_size * (i as f32) + self.sizing.window.x / 2.0)
                        - text_width / 2.0
                        - self.sizing.cell_size * text_displacement_percent,
                    self.sizing.window.y / 2.0 - text_height / 2.0,
                ],
            );

            if i == 0 {
                if let Some(alpha) = self.writing_animation {
                    let write_opacity_square = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::Fill(FillOptions::default()),
                        Rect::new(0.0, 0.0, self.sizing.cell_size, self.sizing.cell_size),
                        Color::new(bg_color.r, bg_color.b, bg_color.g, alpha),
                    )?;

                    canvas.draw(
                        &write_opacity_square,
                        [
                            (self.sizing.cell_size * (i as f32) + self.sizing.window.x / 2.0)
                                - self.sizing.cell_size / 2.0,
                            self.sizing.window.y / 2.0 - self.sizing.cell_size / 2.0,
                        ],
                    );
                }
            }
        }

        // Draw hidden border squares
        let square = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::Fill(FillOptions::default()),
            Rect::new(0.0, 0.0, HORIZ_MARGIN, self.sizing.cell_size + 10.0),
            bg_color,
        )?;
        canvas.draw(
            &square,
            [
                -1.0,
                self.sizing.window.y / 2.0 - (self.sizing.cell_size + 10.0) / 2.0,
            ],
        );
        canvas.draw(
            &square,
            [
                self.sizing.window.x - HORIZ_MARGIN + 1.0,
                self.sizing.window.y / 2.0 - (self.sizing.cell_size + 10.0) / 2.0,
            ],
        );

        let head_square = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::Stroke(StrokeOptions::default().with_line_width(head_stroke_width)),
            Rect::new(0.0, 0.0, self.sizing.cell_size, self.sizing.cell_size),
            ACCENT_COLOR,
        )?;
        canvas.draw(
            &head_square,
            [
                self.sizing.window.x / 2.0 - self.sizing.cell_size / 2.0,
                self.sizing.window.y / 2.0 - self.sizing.cell_size / 2.0,
            ],
        );

        if self.turing_machine.is_halted() {
            let (text_content, text_color) = if self.turing_machine.is_accepting() {
                (
                    "Halted, accepts",
                    if self.light_theme {
                        Color::from([0.0, 0.6, 0.0, 1.0])
                    } else {
                        Color::from_rgb(148, 250, 54)
                    },
                )
            } else {
                ("Halted, rejects", Color::from_rgb(250, 54, 54))
            };

            self.animation_state = None;
            let horiz_text_margin = 20.0;
            let vert_text_margin = 75.0;

            let text_size = 20.0;
            let text_piece = graphics::Text::new(TextFragment {
                text: text_content.to_string(),
                color: Some(text_color),
                scale: Some(PxScale {
                    x: text_size,
                    y: text_size,
                }),
                font: None,
            });
            canvas.draw(&text_piece, [horiz_text_margin, vert_text_margin]);
        }

        {
            let text_margins = 20.0;
            let text_size = 25.0;
            let text_piece = graphics::Text::new(TextFragment {
                text: format!("Running: \"{}\"", self.turing_machine.name()),
                color: Some(fg_color),
                scale: Some(PxScale {
                    x: text_size,
                    y: text_size,
                }),
                font: None,
            });
            canvas.draw(&text_piece, [text_margins, text_margins]);
        }

        {
            let text_margins = 20.0;
            let text_size = 15.0;
            let text_piece = graphics::Text::new(TextFragment {
                text: format!(
                    "Current state: \"{}\"",
                    self.turing_machine.current_state_name()
                ),
                color: Some(fg_color),
                scale: Some(PxScale {
                    x: text_size,
                    y: text_size,
                }),
                font: None,
            });
            canvas.draw(&text_piece, [text_margins, text_margins + 30.0]);
        }

        self.cells_input.draw(ctx, &mut canvas).unwrap();
        self.speed_input.draw(ctx, &mut canvas).unwrap();

        canvas.finish(ctx)?;
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        if self.cells_input.handle_mouse_click(x, y) {
            self.sizing = Sizing::calculate(
                self.sizing.window.x,
                self.sizing.window.y,
                self.cells_input.value() as usize,
            );
        }

        self.speed_input.handle_mouse_click(x, y);
        Ok(())
    }

    fn mouse_motion_event(
        &mut self,
        ctx: &mut Context,
        x: f32,
        y: f32,
        _dx: f32,
        _dy: f32,
    ) -> Result<(), ggez::GameError> {
        set_cursor_type(
            ctx,
            if self.cells_input.is_mouse_over_any_button(x, y)
                || self.speed_input.is_mouse_over_any_button(x, y)
            {
                CursorIcon::Hand
            } else {
                CursorIcon::Default
            },
        );

        Ok(())
    }

    fn resize_event(
        &mut self,
        _ctx: &mut Context,
        width: f32,
        height: f32,
    ) -> Result<(), ggez::GameError> {
        self.sizing = Sizing::calculate(width, height, self.cells_input.value() as usize);

        let mut new_rect = self.cells_input.rect();
        new_rect.y = height - 120.0;
        self.cells_input.set_rect(new_rect);

        let mut new_rect = self.speed_input.rect();
        new_rect.y = height - 50.0;
        self.speed_input.set_rect(new_rect);

        Ok(())
    }
}

pub fn main() -> GameResult {
    let args = args().collect::<Vec<_>>();
    if args.len() < 3 {
        eprintln!("Usage: turing <filename.tng> <tape_data> [--dark]");
        exit(1);
    }

    let dark_theme = args.len() == 4 && args[3] == "--dark";

    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("Turing Machine Simulator", "keelus")
        .add_resource_path(resource_dir);

    const WINDOW_WIDTH: f32 = 1000.0;
    const WINDOW_HEIGHT: f32 = 800.0;

    let (ctx, event_loop) = cb
        .window_mode(
            ggez::conf::WindowMode::default()
                .dimensions(WINDOW_WIDTH, WINDOW_HEIGHT)
                .min_dimensions(400.0, 600.0)
                .resizable(true),
        )
        .window_setup(
            ggez::conf::WindowSetup::default()
                .title("Turing Machine Simulator - by keelus")
                .icon("/icon.png"),
        )
        .build()?;

    let state = MainState::new(&args[1], &args[2], WINDOW_WIDTH, WINDOW_HEIGHT, !dark_theme);
    if let Ok(state) = state {
        event::run(ctx, event_loop, state)
    } else {
        eprintln!("Error: \"{}\"", state.err().unwrap());
        exit(1)
    }
}

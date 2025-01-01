use ggez::{
    graphics::{self, Canvas, Color, Drawable, FillOptions, PxScale, Rect, TextFragment},
    Context, GameResult,
};

pub struct NumberInput {
    rect: Rect,

    label_text: graphics::Text,

    minus_button_rect: Rect,
    plus_button_rect: Rect,

    value: i16,

    increment: i16,
    decrement: i16,

    min: i16,
    max: i16,
}

const MARGIN_VALUE_BUTTON: f32 = 10.0;
const MARGIN_BUTTONS: f32 = 5.0;

impl NumberInput {
    pub fn new(
        label_text: &str,
        start_value: i16,
        increment: i16,
        decrement: i16,
        min: i16,
        max: i16,
        value_rect: Rect,
    ) -> Self {
        let label_text = graphics::Text::new(TextFragment {
            text: label_text.to_string(),
            color: None,
            scale: Some(PxScale { x: 17.0, y: 17.0 }),
            font: None,
        });

        Self {
            label_text,

            rect: value_rect,
            minus_button_rect: Rect::new(
                value_rect.x + value_rect.w + MARGIN_VALUE_BUTTON,
                value_rect.y,
                value_rect.h,
                value_rect.h,
            ),
            plus_button_rect: Rect::new(
                value_rect.x + value_rect.w + MARGIN_VALUE_BUTTON + value_rect.h + MARGIN_BUTTONS,
                value_rect.y,
                value_rect.h,
                value_rect.h,
            ),

            value: start_value,

            increment,
            decrement,

            min,
            max,
        }
    }

    pub fn draw(&self, ctx: &mut Context, canvas: &mut Canvas) -> GameResult {
        // Value rect
        let value_rect = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::Fill(FillOptions::default()),
            self.rect,
            Color::new(0.3, 0.3, 0.3, 1.0),
        )?;
        canvas.draw(&value_rect, [0.0, 0.0]);

        // Value text
        {
            let text_size = 20.0;
            let text_piece = graphics::Text::new(TextFragment {
                text: format!("{}", self.value),
                color: None,
                scale: Some(PxScale {
                    x: text_size,
                    y: text_size,
                }),
                font: None,
            });
            let Rect { h: text_height, .. } = text_piece.dimensions(ctx).unwrap();
            canvas.draw(
                &text_piece,
                [self.rect.x + 5.0, self.rect.y + text_height / 3.0],
            );
        }

        let mut draw_button = |rect: &Rect, text: &str| -> GameResult {
            let button_rectangle = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::Fill(FillOptions::default()),
                *rect,
                Color::RED,
            )?;

            canvas.draw(&button_rectangle, [0.0, 0.0]);
            {
                let text_size = 40.0;
                let text_piece = graphics::Text::new(TextFragment {
                    text: text.to_string(),
                    color: None,
                    scale: Some(PxScale {
                        x: text_size,
                        y: text_size,
                    }),
                    font: None,
                });
                let Rect {
                    w: text_width,
                    h: text_height,
                    ..
                } = text_piece.dimensions(ctx).unwrap();

                let text_x = rect.x - text_width / 2.0 + self.rect.h / 2.0;
                let text_y = rect.y - text_height / 2.0 + self.rect.h / 2.0 + 2.0;
                canvas.draw(&text_piece, [text_x, text_y]);
            }

            Ok(())
        };

        draw_button(&self.minus_button_rect, "-")?;
        draw_button(&self.plus_button_rect, "+")?;

        // Label
        {
            let Rect { h: text_height, .. } = self.label_text.dimensions(ctx).unwrap();

            canvas.draw(
                &self.label_text,
                [self.rect.x - 15.0, self.rect.y - text_height - 5.0],
            );
        }

        Ok(())
    }

    pub fn is_mouse_over_minus_button(&self, x: f32, y: f32) -> bool {
        self.minus_button_rect.contains([x, y])
    }

    pub fn is_mouse_over_plus_button(&self, x: f32, y: f32) -> bool {
        self.plus_button_rect.contains([x, y])
    }

    pub fn is_mouse_over_any_button(&self, x: f32, y: f32) -> bool {
        self.is_mouse_over_minus_button(x, y) || self.is_mouse_over_plus_button(x, y)
    }

    pub fn handle_mouse_click(&mut self, x: f32, y: f32) -> bool {
        if self.is_mouse_over_minus_button(x, y) {
            self.value = (self.value + self.decrement).max(self.min).min(self.max);
            true
        } else if self.is_mouse_over_plus_button(x, y) {
            self.value = (self.value + self.increment).max(self.min).min(self.max);
            true
        } else {
            false
        }
    }

    pub fn value(&self) -> i16 {
        self.value
    }

    pub fn percent(&self) -> f32 {
        (self.value as f32 * 100.0 / self.max as f32) / 100.0
    }

    pub fn rect(&self) -> Rect {
        self.rect
    }

    pub fn set_rect(&mut self, rect: Rect) {
        self.rect = rect;

        self.minus_button_rect
            .move_to([rect.x + rect.w + MARGIN_VALUE_BUTTON, rect.y]);
        self.plus_button_rect.move_to([
            rect.x + rect.w + MARGIN_VALUE_BUTTON + rect.h + MARGIN_BUTTONS,
            rect.y,
        ]);
    }
}

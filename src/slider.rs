use ggez::{
    graphics::{self, Canvas, Color, Drawable, FillOptions, PxScale, Rect, TextFragment},
    mint::Point2,
    Context, GameResult,
};

pub struct Slider {
    rect: Rect,

    handle_radius: f32,

    value: f32,
    being_dragged: bool,
}

impl Slider {
    pub fn new(rect: Rect, handle_radius: f32, initial_value: f32) -> Self {
        Self {
            rect,
            handle_radius,
            value: initial_value,
            being_dragged: false,
        }
    }

    pub fn draw(&self, ctx: &mut Context, canvas: &mut Canvas) -> GameResult {
        let slider_rectangle = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::Fill(FillOptions::default()),
            self.rect,
            Color::new(0.3, 0.3, 0.3, 1.0),
        )?;
        canvas.draw(&slider_rectangle, [0.0, 0.0]);

        // Speed progress
        let slider_progress_rectangle = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::Fill(FillOptions::default()),
            Rect::new(
                self.rect.x,
                self.rect.y,
                self.rect.w * self.value,
                self.rect.h,
            ),
            Color::new(0.8, 0.0, 0.0, 1.0),
        )?;
        canvas.draw(&slider_progress_rectangle, [0.0, 0.0]);

        // Speed progress handle
        let slider_progress_handle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::Fill(FillOptions::default()),
            [0.0, 0.0],
            self.handle_radius,
            1.0,
            Color::new(1.0, 1.0, 1.0, 1.0),
        )?;
        let (handle_x, handle_y) = (
            self.rect.x + self.rect.w * self.value,
            self.rect.y + self.handle_radius / 2.0,
        );
        canvas.draw(&slider_progress_handle, [handle_x, handle_y]);

        // Speed progress text
        {
            let text_fragment = {
                let text_content = format!("{}%", f32::trunc(self.value * 100.0));
                let text_size = 20.0;

                let mut text_fragment = TextFragment::default();
                text_fragment.text = text_content.to_string();
                text_fragment.scale(PxScale {
                    x: text_size,
                    y: text_size,
                })
            };
            let text_piece = graphics::Text::new(text_fragment);
            let Rect { h: text_height, .. } = text_piece.dimensions(ctx).unwrap();

            canvas.draw(
                &text_piece,
                [
                    self.rect.x + self.rect.w + 15.0,
                    self.rect.y + self.rect.h / 2.0 - text_height / 2.0,
                ],
            );
        }

        Ok(())
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn is_mouse_over_handle(&self, x: f32, y: f32) -> bool {
        (self.rect.x + self.rect.w * self.value - x).abs() <= self.handle_radius
            && (self.rect.y + self.rect.h / 2.0 - y).abs() <= self.handle_radius
    }

    pub fn handle_mouse_move(&mut self, x: f32, _y: f32) {
        if !self.being_dragged {
            return;
        }

        let percent = (x - self.rect.x) / self.rect.w;
        let percent = percent.min(1.0).max(0.0);
        self.value = percent;
    }

    pub fn handle_mouse_down(&mut self, x: f32, y: f32) {
        self.being_dragged = self.is_mouse_over_handle(x, y);
    }

    pub fn handle_mouse_up(&mut self, _x: f32, _y: f32) {
        self.being_dragged = false;
    }
}

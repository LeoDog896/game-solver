use egui::{emath, Color32, Pos2, Rect, Rounding, Sense, Ui, Vec2};

use super::Nim;

const CELL_HEIGHT: usize = 10;
const CELL_WIDTH: usize = 5;
const CELL_MARGIN_Y: usize = 5;
const CELL_MARGIN_X: usize = 3;

pub fn display(game: &Nim, ui: &mut Ui) {
    if let Some(max_cell_count) = game.heaps.iter().max() {
        let max_size = Vec2 {
            x: (CELL_WIDTH * max_cell_count + CELL_MARGIN_X * (max_cell_count - 1)) as f32,
            y: (CELL_HEIGHT * game.heaps.len() + CELL_MARGIN_Y * (game.heaps.len() - 1)) as f32
        };

        let (response, painter) =
            ui.allocate_painter(max_size, Sense::drag());

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, max_size),
            response.rect,
        );

        painter.extend(game.heaps.iter().enumerate().map(|(i, heap)| {
            (0..(*heap)).map(|j| {
                egui::Shape::rect_filled(Rect::from_two_pos(
                    to_screen * Pos2 {
                        x: (j * CELL_WIDTH + (j * CELL_MARGIN_X)) as f32,
                        y: (i * CELL_HEIGHT + (i * CELL_MARGIN_Y)) as f32
                    },
                    to_screen * Pos2 {
                        x: (j * CELL_WIDTH + (j * CELL_MARGIN_X) + CELL_WIDTH) as f32,
                        y: (i * CELL_HEIGHT + (i * CELL_MARGIN_Y) + CELL_HEIGHT) as f32
                    }
                ), Rounding::ZERO, Color32::LIGHT_GRAY)
            }).collect::<Vec<_>>()
        }).flatten().collect::<Vec<_>>());
    }
}

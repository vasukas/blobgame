use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

pub trait BevyEguiContextExtended {
    /// Non-decorated fixed UI window.
    /// Anchor specifies positon relatively to the center of the screen, i.e. (-1, -1) is bottom-left.
    /// Optionally can have frame and background.
    fn popup(
        &mut self, name: &str, anchor: Vec2, background: bool, order: egui::Order,
        f: impl FnOnce(&mut egui::Ui),
    );

    /// Fill screen with solid-colored rectangle.
    /// Name must unique among calls in the same frame.
    /// TODO: window size probably can be queried from context, but it's that way for now.
    fn fill_screen(
        &mut self, name: &str, color: egui::Color32, order: egui::Order, window_size: Vec2,
    );
}

impl BevyEguiContextExtended for EguiContext {
    fn popup(
        &mut self, name: &str, anchor: Vec2, background: bool, order: egui::Order,
        f: impl FnOnce(&mut egui::Ui),
    ) {
        let align = |x| {
            if x < 0. {
                egui::Align::Min
            } else if x > 0. {
                egui::Align::Max
            } else {
                egui::Align::Center
            }
        };
        let anchor = egui::Align2([align(anchor.x), align(anchor.y)]);
        egui::Area::new(name)
            .anchor(anchor, egui::vec2(0., 0.))
            .order(order)
            .show(self.ctx_mut(), |ui| {
                if background {
                    egui::Frame::popup(ui.style()).show(ui, f);
                } else {
                    f(ui)
                }
            });
    }

    fn fill_screen(
        &mut self, name: &str, color: egui::Color32, order: egui::Order, window_size: Vec2,
    ) {
        self.popup(name, Vec2::new(0., 0.), false, order, |ui| {
            ui.allocate_space(egui::vec2(window_size.x, window_size.y));
            ui.painter().rect_filled(
                egui::Rect::from_min_max(
                    egui::Pos2::ZERO,
                    egui::pos2(window_size.x, window_size.y),
                ),
                egui::Rounding::none(),
                color,
            );
        });
    }
}

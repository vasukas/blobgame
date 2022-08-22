use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

pub trait BevyEguiContextExtended {
    fn popup(&mut self, name: &str, anchor: Vec2, background: bool, f: impl FnOnce(&mut egui::Ui));
}

impl BevyEguiContextExtended for EguiContext {
    fn popup(&mut self, name: &str, anchor: Vec2, background: bool, f: impl FnOnce(&mut egui::Ui)) {
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
            .show(self.ctx_mut(), |ui| {
                if background {
                    egui::Frame::popup(ui.style()).show(ui, f);
                } else {
                    f(ui)
                }
            });
    }
}

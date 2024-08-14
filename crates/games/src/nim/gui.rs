use egui::Ui;

use crate::util::gui::egui_display::EguiDisplay;

use super::Nim;

impl EguiDisplay for Nim {
    fn display(&self, ui: &mut Ui) {
        for (i, heap) in self.heaps.iter().enumerate() {
            ui.label(format!("Heap {i}: {heap}"));
        }
    }
}

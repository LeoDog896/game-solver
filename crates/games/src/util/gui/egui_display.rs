use egui::Ui;

pub trait EguiDisplay {
    /// Renders
    fn display(&self, ui: &mut Ui);
}

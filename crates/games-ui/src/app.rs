use games::{nim, util::gui::egui_display::EguiDisplay, Games, DEFAULT_GAMES};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
#[derive(Default)]
pub struct TemplateApp {
    /// The currently selected game
    selected_game: Option<Games>,
}

impl TemplateApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load previous app state (if any).
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("game-solver");
            ui.label("Easily solve combinatorial games,\nwithout compromising on performance or interactivity.");

            egui::ComboBox::from_label("Select a game")
                .selected_text(self.selected_game.as_ref().map(|game| game.name()).unwrap_or("No game".to_string()))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.selected_game, None, "No game");
                    for game in DEFAULT_GAMES.iter() {
                        ui.selectable_value(&mut self.selected_game, Some(game.clone()), game.name());
                    }
                });

            ui.separator();

            if let Some(game) = &self.selected_game {
                ui.heading(game.name());
                ui.collapsing("See game description", |ui| {
                    game.description_egui(ui)
                });

                let game = nim::Nim::new(vec![5, 3, 1]);

                game.display(ui);
            } else {
                ui.label("To get started, select a game from the above dropdown.");
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.add(egui::github_link_file!(
                        "https://github.com/LeoDog896/game-solver/blob/master",
                        "Source code."
                    ));
                    powered_by_egui_and_eframe(ui);
                });
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}

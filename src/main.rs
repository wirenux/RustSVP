use eframe::egui;
use std::time::{Duration, Instant};
use std::sync::Arc;

const WPM_MIN: u32 = 10;
const WPM_MAX: u32 = 1000;
const WPM_STEP: u32 = 25;

const WORD_FONT_SIZE: f32 = 80.0;
const PLACEHOLDER_FONT_SIZE: f32 = 40.0;

const GUIDE_BAR_HALF_WIDTH: f32 = 140.0;
const GUIDE_BAR_Y_OFFSET: f32 = 55.0;
const GUIDE_TICK_LENGTH: f32 = 10.0;

fn setup_custom_styles(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "Ubuntu".to_owned(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../assets/Ubuntu-Bold.ttf"
        ))),
    );

    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "Ubuntu".to_owned());

    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "Ubuntu".to_owned());

    ctx.set_fonts(fonts);

    let theme = egui::Theme::Dark;
    let mut style = (*ctx.style_of(theme)).clone();

    style.visuals.panel_fill = egui::Color32::from_rgb(10, 10, 10); // Background color

    style.spacing.button_padding = egui::vec2(16.0, 8.0);
    style.spacing.interact_size.y = 24.0;
    style.spacing.interact_size.x = 70.0;

    let button_rounding = egui::CornerRadius::same(6);

    style.visuals.widgets.inactive.bg_fill = egui::Color32::WHITE;
    style.visuals.widgets.inactive.weak_bg_fill = egui::Color32::WHITE;
    style.visuals.widgets.inactive.corner_radius = button_rounding;
    style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::BLACK);
    style.visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;

    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_gray(230);
    style.visuals.widgets.hovered.weak_bg_fill = egui::Color32::from_gray(230);
    style.visuals.widgets.hovered.corner_radius = button_rounding;
    style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, egui::Color32::BLACK);
    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;

    style.visuals.widgets.active.bg_fill = egui::Color32::from_gray(200);
    style.visuals.widgets.active.weak_bg_fill = egui::Color32::from_gray(200);
    style.visuals.widgets.active.corner_radius = button_rounding;
    style.visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::BLACK);
    style.visuals.widgets.active.bg_stroke = egui::Stroke::NONE;

    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_gray(80);
    style.visuals.widgets.noninteractive.weak_bg_fill = egui::Color32::from_gray(80);
    style.visuals.widgets.noninteractive.corner_radius = button_rounding;
    style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_gray(140));

    ctx.set_style_of(theme, style);
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([700.0, 300.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };

    eframe::run_native(
        "R(ust)SVP",
        options,
        Box::new(|cc| {
            setup_custom_styles(&cc.egui_ctx);
            Ok(Box::<Rsvp>::default())
        }),
    )
}
struct Rsvp {
    words: Vec<String>,
    wpm: u32,
    index: usize,
    last_advance: Instant,
    running: bool,
    show_ui: bool,
    error: Option<String>,
}

const DEMO_TEXT: &str = r#"A black hole is an astronomical body so compact that its gravity prevents anything,
including light, from escaping. Albert Einstein's theory of general relativity,
which describes gravitation as the curvature of spacetime,
predicts that any sufficiently compact mass will form a black hole."#; // Wikipedia Black Hole


impl Default for Rsvp {
    fn default() -> Self {
        Self {
            words: Vec::new(),
            wpm: 300,
            index: 0,
            last_advance: Instant::now(),
            running: false,
            show_ui: true,
            error: None,
        }
    }
}

impl Rsvp {
    fn load_demo(&mut self) {
        self.words = DEMO_TEXT.split_whitespace().map(String::from).collect();
        self.index = 0;
        self.running = false;
        self.last_advance = Instant::now();
    }

    fn load_from_path(&mut self, path: &std::path::Path) {
        if path.extension().and_then(|e| e.to_str()) != Some("txt") {
            self.error = Some(format!("Unsupported file: {}", path.display()));
            return;
        }
        match std::fs::read_to_string(path) {
            Ok(content) => {
                self.words = content.split_whitespace().map(String::from).collect();
                self.index = 0;
                self.running = false;
                self.last_advance = Instant::now();
                self.error = None; // clear any previous error on success
            }
            Err(e) => {
                self.error = Some(format!("Couldn't read file: {e}"));
            }
        }
    }

    fn open_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Text files", &["txt"])
            .pick_file()
        {
            self.load_from_path(&path);
        }
    }

    fn ends_sentence(word: &str) -> bool {
        word.ends_with('.') || word.ends_with('!') || word.ends_with('?')
    }

    fn has_previous_sentence(&self) -> bool {
        self.index > 0
    }

    fn has_next_sentence(&self) -> bool {
        let mut target = self.index + 1;
        while target < self.words.len() {
            let prev_word = &self.words[target - 1];
            if Self::ends_sentence(prev_word) {
                return true;
            }
            target += 1;
        }
        false
    }

    fn jump_to_previous_sentence(&mut self) {
        if self.index == 0 {
            return;
        }

        let mut target = self.index - 1;

        while target > 0 {
            let prev_word = &self.words[target - 1];

            if Self::ends_sentence(prev_word) {
                break;
            }
            target -= 1;
        }

        self.index = target;
        self.last_advance = Instant::now();
    }

    fn jump_to_next_sentence(&mut self) {
        let mut target = self.index + 1;

        while target < self.words.len() {
            let word = &self.words[target - 1];
            
            if Self::ends_sentence(word) {
                break;
            }
            target += 1;
        }

        if target < self.words.len() { // Preventing from going beyond the text length
            self.index = target;
            self.last_advance = Instant::now();
        }
    }

    fn split_orp(word: &str) -> (String, String, String) {
        let chars: Vec<char> = word.chars().collect();
        let char_count = chars.len();

        let clean_len = word
            .trim_end_matches(|c: char| c.is_ascii_punctuation())
            .chars()
            .count();

        let mid_idx = if clean_len > 0 {
            clean_len / 2
        } else {
            char_count / 2
        };

        (
            chars[..mid_idx].iter().collect(),
            chars[mid_idx..mid_idx + 1].iter().collect(),
            chars[mid_idx + 1..].iter().collect(),
        )
    }
}

impl eframe::App for Rsvp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let has_next = self.has_next_sentence();
        let has_previous = self.has_previous_sentence();

        let dropped_path = ui.ctx().input(|i| {
            i.raw.dropped_files.first().map(|f| f.path().to_path_buf())
        });

        if let Some(path) = dropped_path {
            self.load_from_path(&path);
        }

        ui.input(|i| {
            if i.key_pressed(egui::Key::Space) && !self.words.is_empty() {
                self.running = !self.running;
                if self.running {
                    self.last_advance = Instant::now();
                }
            }

            if i.key_pressed(egui::Key::ArrowLeft) && has_previous {
                self.jump_to_previous_sentence();
            }

            if i.key_pressed(egui::Key::ArrowRight) && has_next {
                self.jump_to_next_sentence();
            }

            if i.key_pressed(egui::Key::ArrowUp) {
                self.wpm = (self.wpm.saturating_add(WPM_STEP)).min(WPM_MAX);
            }

            if i.key_pressed(egui::Key::ArrowDown) {
                self.wpm = (self.wpm.saturating_sub(WPM_STEP)).max(WPM_MIN);
            }

            if i.key_pressed(egui::Key::H) {
                self.show_ui = !self.show_ui;
            }

            if i.key_pressed(egui::Key::R) {
                self.index = 0;
                self.running = false;
            }

            if i.key_pressed(egui::Key::D) {
                self.load_demo();
            }

            if i.modifiers.command && i.key_pressed(egui::Key::O) {
                self.open_file();
            }
        });

        egui::CentralPanel::default().show(ui, |ui| {
            if self.show_ui {
                ui.horizontal(|ui| {
                    let start_button_label = if self.running { 
                        "Pause"
                    } else {
                        "Start"
                    };

                    if ui.add_enabled(!self.words.is_empty(), egui::Button::new(start_button_label)).clicked() { // Greyed out if no text
                        self.running = !self.running;
                        if self.running {
                            self.last_advance = Instant::now();
                        }
                    }

                    if ui.add_enabled(has_previous, egui::Button::new("<")).clicked() {
                        self.jump_to_previous_sentence();
                    }

                    if ui.add_enabled(has_next, egui::Button::new(">")).clicked() {
                        self.jump_to_next_sentence();
                    }

                    ui.add(egui::Slider::new(&mut self.wpm, WPM_MIN..=WPM_MAX).text("WPM"));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Reset").clicked() {
                            self.index = 0;
                            self.running = false;
                        }
                        if ui.button("Open...").clicked() {
                            self.open_file();
                        }
                    });
                });
                if let Some(err) = &self.error {
                    ui.colored_label(egui::Color32::from_rgb(220, 80, 80), err);
                }
            }

            let center = ui.available_rect_before_wrap().center();

            if !self.words.is_empty() { // Eye guide line
                let line_color = egui::Color32::from_gray(60);

                // Top
                ui.painter().line_segment(
                    [
                    egui::pos2(center.x - GUIDE_BAR_HALF_WIDTH, center.y -  GUIDE_BAR_Y_OFFSET),
                    egui::pos2(center.x + GUIDE_BAR_HALF_WIDTH, center.y -  GUIDE_BAR_Y_OFFSET),
                    ],
                    egui::Stroke::new(2.0, line_color),
                );
                // Red line
                ui.painter().line_segment(
                    [
                    egui::pos2(center.x, center.y -  GUIDE_BAR_Y_OFFSET),
                    egui::pos2(center.x, center.y -  GUIDE_BAR_Y_OFFSET + GUIDE_TICK_LENGTH),
                    ],
                    egui::Stroke::new(2.0, egui::Color32::RED),
                );

                // Bottom
                ui.painter().line_segment(
                    [
                    egui::pos2(center.x - GUIDE_BAR_HALF_WIDTH, center.y +  GUIDE_BAR_Y_OFFSET),
                    egui::pos2(center.x + GUIDE_BAR_HALF_WIDTH, center.y +  GUIDE_BAR_Y_OFFSET),
                    ],
                    egui::Stroke::new(2.0, line_color),
                );
                // Red line
                ui.painter().line_segment(
                    [
                    egui::pos2(center.x, center.y +  GUIDE_BAR_Y_OFFSET),
                    egui::pos2(center.x, center.y +  GUIDE_BAR_Y_OFFSET - GUIDE_TICK_LENGTH),
                    ],
                    egui::Stroke::new(2.0, egui::Color32::RED),
                );
            }

            if self.words.is_empty() {
                let font_id = egui::FontId::proportional(PLACEHOLDER_FONT_SIZE);
                let galley = ui.painter().layout_no_wrap(
                    "Press Ctrl+O or Cmd+O to open a file.\nOr press D to get the Demo text.".to_string(),
                    font_id,
                    egui::Color32::from_gray(140)
                );
                let pos = egui::pos2(center.x - galley.size().x / 2.0, center.y - galley.size().y / 2.0);
                ui.painter().galley(pos, galley, egui::Color32::from_gray(140));
            } else if let Some(word) = self.words.get(self.index) { // Word rendering
                let (left_part, center_part, right_part) = Self::split_orp(word);

                let font_id = egui::FontId::proportional(WORD_FONT_SIZE);

                let center_galley = ui.painter().layout_no_wrap(
                    center_part,
                    font_id.clone(),
                    egui::Color32::RED,
                );

                let left_galley = ui.painter().layout_no_wrap(
                    left_part,
                    font_id.clone(),
                    egui::Color32::WHITE,
                );

                let right_galley = ui.painter().layout_no_wrap(
                    right_part,
                    font_id.clone(),
                    egui::Color32::WHITE,
                );

                let half_h = center_galley.size().y / 2.0;

                let center_pos = egui::pos2(center.x - (center_galley.size().x / 2.0), center.y - half_h);
                let left_pos = egui::pos2(center_pos.x - left_galley.size().x, center.y - half_h);
                let right_pos = egui::pos2(center_pos.x + center_galley.size().x, center.y - half_h);

                ui.painter().galley(left_pos, left_galley, egui::Color32::WHITE);
                ui.painter().galley(center_pos, center_galley, egui::Color32::RED);
                ui.painter().galley(right_pos, right_galley, egui::Color32::WHITE);
            }

            if self.running {
                let interval = Duration::from_secs_f32(60.0 / self.wpm as f32);
                if self.last_advance.elapsed() >= interval {
                    if self.index + 1 < self.words.len() {
                        self.index += 1;
                        self.last_advance = Instant::now();
                    } else {
                        self.running = false;
                    }
                }
                ui.ctx().request_repaint();
            }
            if !ui.ctx().input(|i| i.raw.hovered_files.is_empty()) { // Drew last so on top
                let screen_rect = ui.ctx().content_rect();
                ui.painter().rect_filled(
                    screen_rect,
                    0.0,
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180),
                );
                ui.painter().text(
                    screen_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Drop .txt file to load",
                    egui::FontId::proportional(30.0),
                    egui::Color32::WHITE,
                );
            }
        });
    }
}

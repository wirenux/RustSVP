use eframe::egui;
use std::time::{Duration, Instant};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native(
        "R(ust)SVP",
        options,
        Box::new(|_cc| {
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
}

impl Default for Rsvp {
    fn default() -> Self {
        let text = r#"A black hole is an astronomical body so compact that its gravity prevents anything,
including light, from escaping. Albert Einstein's theory of general relativity,
which describes gravitation as the curvature of spacetime,
predicts that any sufficiently compact mass will form a black hole."#; // Wikipedia Black Hole

        Self {
            words: text
                .split_whitespace()
                .map(String::from)
                .collect(),
            wpm: 300,
            index: 0,
            last_advance: Instant::now(),
            running: false,
        }
    }
}

impl Rsvp {
    fn jump_to_previous_sentence(&mut self) {
        if self.index == 0 {
            return;
        }

        let mut target = self.index - 1;

        while target > 0 {
            let word = &self.words[target - 1];

            if word.ends_with('.') || word.ends_with('!') || word.ends_with('?') {
                break;
            }
            target -= 1;
        }

        self.index = target;
        self.last_advance = Instant::now();
    }
}

impl eframe::App for Rsvp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.add(egui::Slider::new(&mut self.wpm, 10..=1000).text("WPM"));

            let start_button_label = if self.running { 
                "Pause"
            } else {
                "Start"
            };

            if ui.button(start_button_label).clicked() {
                self.running = !self.running;
                if self.running {
                    self.last_advance = Instant::now();
                }
            }

            if ui.button("Back").clicked() {
                self.jump_to_previous_sentence();
            }

            if ui.button("Reset").clicked() {
                self.index = 0;
                self.running = false;
            }

            if let Some(word) = self.words.get(self.index) {
                let chars: Vec<char> = word.chars().collect();
                let char_count = word.chars().count();

                if char_count > 0 {
                    let clean_len = word
                        .trim_end_matches(|c: char| c.is_ascii_punctuation())
                        .chars()
                        .count();

                    let mid_idx = if clean_len > 0 {
                        clean_len / 2
                    } else {
                        char_count / 2
                    };

                    let left: String = chars[..mid_idx].iter().collect();
                    let center: String = chars[mid_idx..mid_idx + 1].iter().collect();
                    let right: String = chars[mid_idx + 1..].iter().collect();

                    let center_x = ui.available_rect_before_wrap().center().x;
                    let center_y = ui.available_rect_before_wrap().center().y;

                    let font_id = egui::FontId::proportional(48.0);

                    let center_galley = ui.painter().layout_no_wrap(
                        center,
                        font_id.clone(),
                        egui::Color32::RED,
                    );

                    let left_galley = ui.painter().layout_no_wrap(
                        left,
                        font_id.clone(),
                        egui::Color32::WHITE,
                    );

                    let right_galley = ui.painter().layout_no_wrap(
                        right,
                        font_id.clone(),
                        egui::Color32::WHITE,
                    );

                    let center_pos = egui::pos2(center_x - (center_galley.size().x / 2.0 ), center_y);
                    let left_pos = egui::pos2(center_pos.x - left_galley.size().x, center_y);
                    let right_pos = egui::pos2(center_pos.x + center_galley.size().x, center_y);

                    ui.painter().galley(left_pos, left_galley, egui::Color32::WHITE);
                    ui.painter().galley(center_pos, center_galley, egui::Color32::RED);
                    ui.painter().galley(right_pos, right_galley, egui::Color32::WHITE);
                }
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
            }

            ui.ctx().request_repaint();
        });
    }
}

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
}

impl Default for Rsvp {
    fn default() -> Self {
        let text = r#"A black hole is an astronomical body so compact that its gravity prevents anything,
including light, from escaping. Albert Einstein's theory of general relativity,
which describes gravitation as the curvature of spacetime,
predicts that any sufficiently compact mass will form a black hole."#;

        Self {
            words: text.split_whitespace().map(String::from).collect(),
            wpm: 10,
            index: 0,
            last_advance: Instant::now(),
        }
    }
}

impl eframe::App for Rsvp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.add(egui::Slider::new(&mut self.wpm, 10..=1000).text("WPM"));

            if let Some(word) = self.words.get(self.index) {
                ui.label(word);
            }

            let interval = Duration::from_secs_f32(60.0 / self.wpm as f32);
            if self.index < self.words.len() && self.last_advance.elapsed() >= interval {
                self.index += 1;
                self.last_advance = Instant::now();
            }
        });

        ui.ctx().request_repaint();
    }
}

use std::time::{Duration, Instant};

use eframe::{
    egui::{self, Button, RichText, ViewportBuilder},
    epaint::Color32,
};

fn main() {
    let viewport_builder = ViewportBuilder::default()
        .with_resizable(false)
        .with_inner_size((300.0, 400.0))
        .with_position((0.0, 0.0));

    let native_options = eframe::NativeOptions {
        viewport: viewport_builder,
        ..eframe::NativeOptions::default()
    };
    eframe::run_native(
        "CrabSplit",
        native_options,
        Box::new(|cc| Box::new(CrabSplit::new(cc))),
    )
    .unwrap();
}

struct Task {
    name: String,
    duration: Duration,
    started_at: Option<Instant>,
}

#[derive(Default)]
struct CrabSplit {
    current_task: usize,
    task_name: String,
    tasks: Vec<Task>,
    running: bool,
}

impl CrabSplit {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self {
            current_task: 0,
            task_name: "".to_string(),
            tasks: Vec::with_capacity(10),
            running: false,
        }
    }

    fn add_task(&mut self) {
        if self.task_name.len() > 0 {
            let task = Task {
                name: self.task_name.clone(),
                duration: Duration::new(0, 0),
                started_at: None,
            };
            self.tasks.push(task);
            self.task_name.clear();
        }
    }

    fn next_task(&mut self) {
        self.current_task += 1;
    }

    fn start(&mut self) {
        self.running = true;
        self.tasks[self.current_task].started_at = Some(Instant::now());
    }

    fn stop(&mut self) {
        self.running = false;
        let now = Instant::now();
        let elapsed = now
            - self.tasks[self.current_task]
                .started_at
                .expect("This has to be Some because it is currently running");

        self.tasks[self.current_task].started_at = None;
        self.tasks[self.current_task].duration += elapsed;
    }

    fn start_enabled(&self) -> bool {
        self.running == false && self.tasks.len() > 0
    }

    fn stop_enabled(&self) -> bool {
        self.running == true
    }

    fn next_task_enabled(&self) -> bool {
        self.running == false && self.tasks.len() > 0 && self.current_task < self.tasks.len() - 1
    }

    fn calculate_total_time(&self) -> String {
        let mut total = Duration::new(0, 0);
        for task in &self.tasks {
            total += task.duration;
        }

        if self.running {
            let current_task = &self.tasks[self.current_task];

            let now = Instant::now();
            let elapsed = now - current_task.started_at.unwrap();
            total += elapsed;
        }

        format!("{:?}", total)
    }
}

impl eframe::App for CrabSplit {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(self.calculate_total_time());

            ui.vertical(|ui| {
                for (idx, task) in self.tasks.iter().enumerate() {
                    let text = if let Some(started_at) = task.started_at {
                        let now = Instant::now();
                        let elapsed = now - started_at;
                        format!("{} - {:?}", task.name, task.duration + elapsed)
                    } else {
                        format!("{} - {:?}", task.name, task.duration)
                    };

                    if idx == self.current_task {
                        ui.label(RichText::new(text).color(Color32::from_rgb(0, 255, 0)));
                    } else {
                        ui.label(RichText::new(text).color(Color32::from_rgb(255, 255, 255)));
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.task_name);
            });
            ui.horizontal(|ui| {
                if ui.button("Add Task").clicked() {
                    self.add_task();
                };
                if ui
                    .add_enabled(self.next_task_enabled(), Button::new("Next Task"))
                    .clicked()
                {
                    self.next_task();
                };
                if ui
                    .add_enabled(self.start_enabled(), Button::new("Start"))
                    .clicked()
                {
                    self.start();
                };
                if ui
                    .add_enabled(self.stop_enabled(), Button::new("Stop"))
                    .clicked()
                {
                    self.stop();
                };
            });
        });
    }
}

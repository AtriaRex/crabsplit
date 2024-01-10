#![windows_subsystem = "windows"]

use std::{
    fmt::Display,
    fs::{self, File, OpenOptions},
    str::FromStr,
    time::{Duration, SystemTime},
};

use std::io::Write;

use chrono::{Date, DateTime, Datelike, TimeZone, Timelike, Utc};
use eframe::{
    egui::{self, Button, RichText, ViewportBuilder},
    epaint::Color32,
};
use serde::{Deserialize, Serialize};
use tzfile::RcTz;

#[cfg(target_os = "linux")]
const DEFAULT_PATH: &'static str = "/home/emre/crabsplit";
#[cfg(target_os = "windows")]
static DEFAULT_PATH: &'static str = "C:/Users/aliem";

fn main() {
    let viewport_builder = ViewportBuilder::default()
        .with_resizable(false)
        .with_inner_size((300.0, 400.0))
        .with_position((0.0, 0.0));

    let native_options = eframe::NativeOptions {
        viewport: viewport_builder,
        ..eframe::NativeOptions::default()
    };

    let tasks = read_today();
    eframe::run_native(
        "CrabSplit",
        native_options,
        Box::new(|cc| Box::new(CrabSplit::new(cc, tasks))),
    )
    .unwrap();
}

#[derive(Serialize, Deserialize)]
struct TaskProgress {
    pub start: SystemTime,
    pub end: SystemTime,
}

fn to_datetime(system_time: SystemTime) -> DateTime<RcTz> {
    let timestamp = system_time
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let tz: RcTz = RcTz::named("Europe/Istanbul").unwrap();
    let datetime = DateTime::from_timestamp(timestamp.try_into().unwrap(), 0)
        .unwrap()
        .with_timezone(&tz);

    datetime
}

impl Display for TaskProgress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let start = to_datetime(self.start);
        let end = to_datetime(self.end);

        write!(
            f,
            "{}:{} - {}:{}",
            start.hour(),
            start.minute(),
            end.hour(),
            end.minute()
        )
    }
}

impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.name)?;
        for (idx, progress) in self.progress.iter().enumerate() {
            if idx == self.progress.len() {
                write!(f, "  {}", progress)?;
            } else {
                writeln!(f, "  {}", progress)?;
            }
        }

        Ok(())
    }
}

// impl FromStr for TaskProgress {
//     fn from_str(s: &str) -> Result<Self, Self::Err> {}
// }

#[derive(Serialize, Deserialize)]
struct Task {
    name: String,
    progress: Vec<TaskProgress>,
    started_at: Option<SystemTime>,
}

#[derive(Default)]
struct CrabSplit {
    current_task: usize,
    task_name: String,
    tasks: Vec<Task>,
    running: bool,
}

impl CrabSplit {
    fn new(cc: &eframe::CreationContext<'_>, tasks: Option<Vec<Task>>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        if let Some(tasks) = tasks {
            Self {
                current_task: 0,
                task_name: "".to_string(),
                running: false,
                tasks,
            }
        } else {
            Self {
                current_task: 0,
                task_name: "".to_string(),
                running: false,
                tasks: Vec::with_capacity(10),
            }
        }
    }

    fn add_task(&mut self) {
        if self.task_name.len() > 0 {
            let task = Task {
                name: self.task_name.clone(),
                progress: Vec::with_capacity(10),
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
        self.tasks[self.current_task].started_at = Some(SystemTime::now());
    }

    fn stop(&mut self) {
        self.running = false;
        let task = &self.tasks[self.current_task];
        let task_progress = TaskProgress {
            start: task.started_at.unwrap(),
            end: SystemTime::now(),
        };
        self.tasks[self.current_task].started_at = None;
        self.tasks[self.current_task].progress.push(task_progress);
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

    fn calculate_task_elapsed(task: &Task) -> Duration {
        let mut total = Duration::new(0, 0);
        for progress in &task.progress {
            let duration = progress.end.duration_since(progress.start).unwrap();
            total += duration;
        }

        if let Some(started_at) = task.started_at {
            let now = SystemTime::now();
            let elapsed = now.duration_since(started_at).unwrap();
            total += elapsed;
        }

        total
    }

    fn calculate_total_elapsed(&self) -> Duration {
        let mut total = Duration::new(0, 0);

        for task in &self.tasks {
            total += Self::calculate_task_elapsed(task);
        }

        total
    }
}

fn record_today(tasks: &Vec<Task>) {
    let tz: RcTz = RcTz::named("Europe/Istanbul").unwrap();
    let today = Utc::now().with_timezone(&tz);
    let filename = format!("{}-{}-{}", today.day(), today.month(), today.year());

    let tasks_str = serde_json::to_string(tasks).unwrap();

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(format!("{DEFAULT_PATH}/{filename}"))
        .unwrap();

    writeln!(file, "{}", tasks_str).unwrap();
}

fn read_today() -> Option<Vec<Task>> {
    let tz: RcTz = RcTz::named("Europe/Istanbul").unwrap();
    let today = Utc::now().with_timezone(&tz);
    let filename = format!("{}-{}-{}", today.day(), today.month(), today.year());

    let file = fs::read_to_string(format!("{DEFAULT_PATH}/{filename}"));

    match file {
        Ok(file) => Some(serde_json::from_str(&file).unwrap()),
        Err(e) => None,
    }
}

impl eframe::App for CrabSplit {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint();

        if ctx.input(|i| i.viewport().close_requested()) {
            // write tasks to file and close
            record_today(&self.tasks)
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(format!("{:?}", self.calculate_total_elapsed()));

            ui.vertical(|ui| {
                for (idx, task) in self.tasks.iter().enumerate() {
                    let task_duration = Self::calculate_task_elapsed(task);
                    let text = format!("{} - {:?}", task.name, task_duration);

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

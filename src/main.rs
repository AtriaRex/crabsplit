#![windows_subsystem = "windows"]

use std::{
    fmt::Display,
    fs::{self, OpenOptions},
    thread,
    time::{Duration, SystemTime},
};

use chrono_tz::{Europe::Istanbul, Tz};

use std::io::Write;

use chrono::{DateTime, Datelike, Timelike, Utc};
use eframe::{
    egui::{self, Button, RichText, ViewportBuilder},
    epaint::Color32,
};
use serde::{Deserialize, Serialize};
use std::path::{PathBuf, Path};


fn main() {
    let home_dir = std::env::var("HOME").unwrap();
    let home_dir = Path::new(&home_dir);
    let crabsplit_dir = home_dir.join("crabsplit");
    
    let viewport_builder = ViewportBuilder::default()
        .with_resizable(false)
        .with_inner_size((300.0, 400.0))
        .with_position((0.0, 0.0));

    let native_options = eframe::NativeOptions {
        viewport: viewport_builder,
        ..eframe::NativeOptions::default()
    };

    let today = Utc::now().with_timezone(&Istanbul);
    let filename = format!("{}-{}-{}", today.day(), today.month(), today.year());
    let tasks = read_today(&filename, &crabsplit_dir);

    eframe::run_native(
        "CrabSplit",
        native_options,
        Box::new(|cc| Box::new(CrabSplit::new(cc, tasks, filename, crabsplit_dir))),
    )
    .unwrap();
}

#[derive(Serialize, Deserialize)]
struct TaskProgress {
    pub start: SystemTime,
    pub end: SystemTime,
}

fn to_datetime(system_time: SystemTime) -> DateTime<Tz> {
    let timestamp = system_time
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let datetime = DateTime::from_timestamp(timestamp.try_into().unwrap(), 0)
        .unwrap()
        .with_timezone(&Istanbul);

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
    filename: String,
    dir: PathBuf,
}

impl CrabSplit {
    fn new(cc: &eframe::CreationContext<'_>, tasks: Option<Vec<Task>>, filename: String, dir: PathBuf) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        let repainter = cc.egui_ctx.clone();
        thread::spawn(move || loop {
            std::thread::sleep(Duration::new(1, 0));
            repainter.request_repaint();
        });

        if let Some(tasks) = tasks {
            Self {
                current_task: 0,
                task_name: "".to_string(),
                running: false,
                tasks,
                filename,
                dir,
            }
        } else {
            Self {
                current_task: 0,
                task_name: "".to_string(),
                running: false,
                tasks: Vec::with_capacity(10),
                filename,
                dir,
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

    fn remove_task(&mut self, index: usize) {
        self.tasks.remove(index);
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

        Self::truncate_duration(total)
    }

    fn truncate_duration(duration: Duration) -> Duration {
        let seconds = duration.as_secs();
        Duration::from_secs(seconds)
    }

    fn calculate_total_elapsed(&self) -> Duration {
        let mut total = Duration::new(0, 0);

        for task in &self.tasks {
            total += Self::calculate_task_elapsed(task);
        }

        total
    }

    fn record_today(&self) {
        let tasks_str = serde_json::to_string(&self.tasks).unwrap();
        let filename = &self.filename;

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(format!("{}/{}", self.dir.to_str().unwrap(), filename))
            .unwrap();

        writeln!(file, "{}", tasks_str).unwrap();
    }
}

fn read_today(filename: &str, dir: &PathBuf) -> Option<Vec<Task>> {
    let file = fs::read_to_string(format!("{}/{}", dir.to_str().unwrap(), filename));

    match file {
        Ok(file) => Some(serde_json::from_str(&file).unwrap()),
        Err(e) => None,
    }
}

fn format_duration(duration: &Duration) -> String {
    let total_seconds = duration.as_secs();

    let full_minutes = total_seconds / 60;
    let seconds = total_seconds % 60;

    format!("{}:{}", full_minutes, seconds)
}

impl eframe::App for CrabSplit {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if ctx.input(|i| i.viewport().close_requested()) {
            // if any task is running stop it.
            if self.running {
                self.stop();
            }

            // write tasks to file and close
            self.record_today();
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            self.add_task();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(format!(
                "{}",
                format_duration(&self.calculate_total_elapsed())
            ));

            ui.vertical(|ui| {
                let mut task_to_remove: Option<usize> = None;

                for (idx, task) in self.tasks.iter().enumerate() {
                    let task_duration = Self::calculate_task_elapsed(task);
                    let text = format!("{} - {}", task.name, format_duration(&task_duration));

                    if idx == self.current_task {
                        if self.running {
                            ui.horizontal_wrapped(|ui| {
                                ui.label(RichText::new(text).color(Color32::from_rgb(255, 0, 0)));
                                if ui.button("X").clicked() {
                                    task_to_remove = Some(idx);
                                }
                            });
                        } else {
                            ui.horizontal_wrapped(|ui| {
                                ui.label(RichText::new(text).color(Color32::from_rgb(0, 255, 0)));
                                if ui.button("X").clicked() {
                                    task_to_remove = Some(idx);
                                }
                            });
                        }
                    } else {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new(text).color(Color32::from_rgb(255, 255, 255)));
                            if ui.button("X").clicked() {
                                task_to_remove = Some(idx);
                            }
                        });
                    }
                }

                if let Some(task_to_remove) = task_to_remove {
                    self.remove_task(task_to_remove);
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

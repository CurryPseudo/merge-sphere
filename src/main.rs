#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use std::io::Cursor;

use eframe::epi;
use egui::{panel::Side, DragValue, Label, Slider};
use egui_extras::RetainedImage;
use image::{Rgb, RgbImage};
use imageproc::drawing::*;

struct App {
    width: i32,
    height: i32,
    image: Option<RetainedImage>,
    circles: [(i32, i32, i32); 3],
    show_first_merge: bool,
    show_second_merge: bool,
}

impl App {
    fn build_image(&mut self) {
        let mut image = RgbImage::new(self.width as u32, self.height as u32);
        let circles = self.circles;
        for (x, y, radius) in circles {
            draw_hollow_circle_mut(&mut image, (x, y), radius, Rgb([0, 0, 255]));
        }

        fn merge_sphere(
            (mut x0, mut y0, mut radius0): (f32, f32, f32),
            (mut x1, mut y1, mut radius1): (f32, f32, f32),
        ) -> (f32, f32, f32) {
            if radius0 > radius1 {
                std::mem::swap(&mut x0, &mut x1);
                std::mem::swap(&mut y0, &mut y1);
                std::mem::swap(&mut radius0, &mut radius1);
            }
            let (dx, dy) = (x1 - x0, y1 - y0);
            let len_d = (dx * dx + dy * dy).sqrt();
            let radius = if len_d + radius0 <= radius1 {
                return (x1, y1, radius1);
            } else {
                (len_d + radius0 + radius1) / 2.
            };
            let (d_unit_x, d_unit_y) = (dx / len_d, dy / len_d);
            let (x, y) = (
                x0 + d_unit_x * (radius - radius0),
                y0 + d_unit_y * (radius - radius0),
            );
            (x, y, radius)
        }

        for (first, second, last) in [
            (0, 1, 2),
            (0, 2, 1),
            (1, 0, 2),
            (1, 2, 0),
            (2, 0, 1),
            (2, 1, 0),
        ] {
            let (x, y, radius) = merge_sphere(
                (
                    circles[first].0 as f32,
                    circles[first].1 as f32,
                    circles[first].2 as f32,
                ),
                (
                    circles[second].0 as f32,
                    circles[second].1 as f32,
                    circles[second].2 as f32,
                ),
            );
            if self.show_first_merge {
                draw_hollow_circle_mut(
                    &mut image,
                    (x as i32, y as i32),
                    radius as i32,
                    Rgb([0, 255, 0]),
                );
            }
            let (x, y, radius) = merge_sphere(
                (x, y, radius),
                (
                    circles[last].0 as f32,
                    circles[last].1 as f32,
                    circles[last].2 as f32,
                ),
            );
            if self.show_second_merge {
                draw_hollow_circle_mut(
                    &mut image,
                    (x as i32, y as i32),
                    radius as i32,
                    Rgb([255, 0, 0]),
                )
            }
        }
        let mut buffer = Vec::new();
        image
            .write_to(&mut Cursor::new(&mut buffer), image::ImageOutputFormat::Png)
            .unwrap();
        self.image = Some(RetainedImage::from_image_bytes("Merge Sphere", &buffer).unwrap());
    }
    fn new(width: i32, height: i32) -> Self {
        let mut r = Self {
            image: None,
            circles: [(150, 150, 100), (350, 130, 30), (120, 330, 80)],
            width,
            height,
            show_first_merge: true,
            show_second_merge: false,
        };
        r.build_image();
        r
    }
}

impl epi::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        let mut should_build_image = false;
        egui::SidePanel::new(Side::Left, "Circle Config").show(ctx, |ui| {
            for (i, (x, y, radius)) in self.circles.iter_mut().enumerate() {
                ui.add(Label::new(format!("Circle {}", i)));
                ui.indent("Circle", |ui| {
                    ui.horizontal(|ui| {
                        ui.add(Label::new("x"));
                        if ui.add(Slider::new(x, 0..=self.width)).changed() {
                            should_build_image = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.add(Label::new("y"));
                        if ui.add(Slider::new(y, 0..=self.height)).changed() {
                            should_build_image = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.add(Label::new("radius"));
                        if ui.add(DragValue::new(radius)).changed() {
                            should_build_image = true;
                        }
                    });
                });
            }
            ui.separator();
            ui.horizontal(|ui| {
                ui.add(Label::new("show first merge"));
                if ui
                    .add(egui::Checkbox::new(&mut self.show_first_merge, ""))
                    .changed()
                {
                    should_build_image = true;
                }
            });
            ui.horizontal(|ui| {
                ui.add(Label::new("show second merge"));
                if ui
                    .add(egui::Checkbox::new(&mut self.show_second_merge, ""))
                    .changed()
                {
                    should_build_image = true;
                }
            });
        });
        if should_build_image {
            self.build_image();
        };
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(image) = &self.image {
                image.show(ui);
            }
        });
        frame.set_window_size(ctx.used_size());
    }

    fn name(&self) -> &str {
        "Merge sphere"
    }
}

fn main() {
    let width = 500;
    let height = 500;
    let options = eframe::NativeOptions {
        resizable: false,
        ..Default::default()
    };
    eframe::run_native(Box::new(App::new(width, height)), options);
}

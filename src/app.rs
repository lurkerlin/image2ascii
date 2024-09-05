use std::{future::Future, sync::mpsc::{Receiver, Sender}};
use crate::ascii::image_to_ascii;
use eframe::App;
use egui::{FontData, FontDefinitions, FontFamily, TextureHandle};
use image::DynamicImage;

pub struct MyApp {
    image_channel: (Sender<DynamicImage>, Receiver<DynamicImage>),
    image: DynamicImage,
    ascii_art: String,
    width: u32,
    original_texture: Option<TextureHandle>, // 原始图像的纹理
    grayscale_texture: Option<TextureHandle>, // 灰度图像的纹理
    resized_texture: Option<TextureHandle>,  // 调整大小后的图像的纹理
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            image_channel: std::sync::mpsc::channel(),
            image: image::DynamicImage::new_rgb8(1, 1),
            ascii_art: String::new(),
            width: 100,
            original_texture: None,
            grayscale_texture: None,
            resized_texture: None,
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // #[cfg(not(target_arch = "wasm32"))]
            // if ui.button("Load Image").clicked() {
            //     if let Some(file_path) = rfd::FileDialog::new().pick_file() {
            //         let file_data = std::fs::read(file_path).unwrap();
            //         self.load_image(&file_data, &ctx);
            //     }
            // }
            if let Ok(img) = self.image_channel.1.try_recv() {
                self.load_image(img, ctx);
            }

            // #[cfg(target_arch = "wasm32")]
            if ui.button("📂 Open image file").clicked() {
                let sender = self.image_channel.0.clone();
                let task = rfd::AsyncFileDialog::new().pick_file();
                // Context is wrapped in an Arc so it's cheap to clone as per:
                // > Context is cheap to clone, and any clones refers to the same mutable data (Context uses refcounting internally).
                // Taken from https://docs.rs/egui/0.24.1/egui/struct.Context.html
                let ctx = ui.ctx().clone();
                execute(async move {
                    let file = task.await;
                    if let Some(file) = file {
                        let bytes = file.read().await;
                        let _ = sender.send(image::load_from_memory(&bytes).unwrap());
                        ctx.request_repaint();
                    }
                });
            }
            // Slider
            ui.add(egui::Slider::new(&mut self.width, 10..=200).text("Width"));
            self.ascii_art = self.imaeg2ascii(self.width);

            ui.horizontal(|ui| {
                ui.monospace(&self.ascii_art);

                // 右边显示图像处理过程
                ui.vertical(|ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.push_id("original_image", |ui| {
                            if let Some(texture) = &self.original_texture {
                                ui.label("Original Image:");
                                ui.image(texture);
                            }
                        });
                        ui.push_id("grayscale_image", |ui| {
                            if let Some(texture) = &self.grayscale_texture {
                                ui.label("Grayscale Image:");
                                ui.image(texture);
                            }
                        });

                        ui.push_id("resized_image", |ui| {
                            if let Some(texture) = &self.resized_texture {
                                ui.label("Resized Image:");
                                ui.image(texture);
                            }
                        });
                    });
                });
            });
        });
    }
}

impl MyApp {
    pub fn setup_fonts(&mut self, ctx: &egui::Context) {
        let mut fonts = FontDefinitions::default();

        fonts.font_data.insert(
            "Firacode".to_string(),
            FontData::from_static(include_bytes!("../Fira Mono-Regular.ttf")),
        );
        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .insert(0, "Firacode".to_string());

        ctx.set_style(egui::style::Style::default());
        ctx.set_fonts(fonts);
    }
    fn load_image(&mut self,image:DynamicImage, ctx: &egui::Context) {
        self.image = image.clone();
        // 显示原始图像
        let size = [image.width() as usize, image.height() as usize];
        let raw_image = egui::ColorImage::from_rgba_unmultiplied(size, &self.image.to_rgba8());
        self.original_texture =
            Some(ctx.load_texture("original_image", raw_image, Default::default()));

        // 生成灰度图像
        let grayscale_img = self.image.grayscale();
        let grayscale_size = [
            grayscale_img.width() as usize,
            grayscale_img.height() as usize,
        ];
        let grayscale_image =
            egui::ColorImage::from_rgba_unmultiplied(grayscale_size, &grayscale_img.to_rgba8());
        self.grayscale_texture =
            Some(ctx.load_texture("grayscale_image", grayscale_image, Default::default()));

        // 调整大小
        let resized_img =
            grayscale_img.resize_exact(100, 100, image::imageops::FilterType::Nearest);
        let resized_size = [resized_img.width() as usize, resized_img.height() as usize];
        let resized_image =
            egui::ColorImage::from_rgba_unmultiplied(resized_size, &resized_img.to_rgba8());
        self.resized_texture =
            Some(ctx.load_texture("resized_image", resized_image, Default::default()));
    }

    fn imaeg2ascii(&self, width: u32) -> String {
        image_to_ascii(&self.image, width)
    }
}

// #[cfg(target_arch = "wasm32")]
fn execute<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}
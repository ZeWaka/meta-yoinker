use egui::{vec2, FontFamily, FontId, RichText, TextStyle};
use egui_extras::RetainedImage;
use std::io::Cursor;

pub struct MetadataApp {
	img: Option<RetainedImage>,
	img_offset: egui::Pos2,
	dropped_files: Vec<egui::DroppedFile>,
	image_info: Option<FileInfo>,

	available_height: f32,
	available_width: f32,
}

#[derive(Default)]
struct FileInfo {
	name: String,
	path: String,
	extension: String,
	bytes: usize,
}

impl Default for MetadataApp {
	fn default() -> Self {
		Self {
			img: None,
			image_info: None,
			available_height: 0.0,
			available_width: 0.0,
			img_offset: egui::pos2(0.0, 0.0),
			dropped_files: Vec::default(),
		}
	}
}

fn configure_text_styles(ctx: &egui::Context) {
	use FontFamily::{Monospace, Proportional};

	let mut style = (*ctx.style()).clone();
	style.text_styles = [
		(TextStyle::Heading, FontId::new(25.0, Proportional)),
		(heading2(), FontId::new(22.0, Proportional)),
		(heading3(), FontId::new(19.0, Proportional)),
		(TextStyle::Body, FontId::new(16.0, Proportional)),
		(TextStyle::Monospace, FontId::new(12.0, Monospace)),
		(TextStyle::Button, FontId::new(12.0, Proportional)),
		(TextStyle::Small, FontId::new(8.0, Proportional)),
	]
	.into();
	ctx.set_style(style);
}

#[inline]
fn heading2() -> TextStyle {
	TextStyle::Name("Heading2".into())
}

#[inline]
fn heading3() -> TextStyle {
	TextStyle::Name("ContextHeading".into())
}

impl MetadataApp {
	/// Called once before the first frame.
	pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
		// This is also where you can customize the look and feel of egui using
		// `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
		configure_text_styles(&cc.egui_ctx);
		Default::default()
	}

	fn preview_files_being_dropped(&mut self, ctx: &egui::Context) {
		use egui::*;

		// Preview hovering files:
		if ctx.input(|i| i.raw.hovered_files.is_empty()) {
			return;
		}
		let text = ctx.input(|i| {
			let mut text = "Dropping files:\n".to_owned();
			for file in &i.raw.hovered_files {
				if let Some(path) = &file.path {
					text += &format!("\n{}", path.display());
				} else if !file.mime.is_empty() {
					text += &format!("\n{}", file.mime);
				} else {
					text += "\n???";
				}
			}
			text
		});

		let painter =
			ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

		let screen_rect = ctx.screen_rect().shrink(10.0);
		painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(70));
		painter.text(
			screen_rect.center(),
			Align2::CENTER_CENTER,
			text,
			TextStyle::Heading.resolve(&ctx.style()),
			Color32::WHITE,
		);
	}

	fn load_file_or_err(&mut self, ui: &mut egui::Ui) {
		if !self.dropped_files.is_empty() {
			ui.group(|ui| {
				ui.label("Dropped files:");

				for file in &self.dropped_files {
					let mut info = FileInfo::default();
					if let Some(path) = &file.path {
						info.path = path.display().to_string()
					} else if !file.name.is_empty() {
						let filesplit: Vec<&str> = file.name.split('.').collect(); // yes i know this is terrible
						info.name = filesplit[0].to_owned();
						info.extension = filesplit[1].to_owned();
					} else {
						info.name = "???".to_owned()
					};
					if let Some(bytes) = &file.bytes {
						info.bytes = bytes.len();
						if self.img.is_none() {
							let mut buffer: Vec<u8> = Vec::new();
							let mut writer = Cursor::new(&mut buffer);

							let mut i = match image::load_from_memory_with_format(
								bytes,
								image::ImageFormat::Png,
							) {
								Ok(image) => image,
								Err(e) => {
									ui.colored_label(
										egui::Color32::RED,
										format!("Error loading {} from memory: {e}", file.name),
									);
									return;
								}
							};

							let h = (self.available_height - 10.0) as u32;
							let w = (self.available_width - 10.0) as u32;

							ui.heading(format!("height {}", h));
							i = i.resize(w, w, image::imageops::FilterType::Nearest);
							i.write_to(&mut writer, image::ImageFormat::Png).unwrap();

							self.img = None;
							self.img =
								Some(RetainedImage::from_image_bytes("img", &buffer).unwrap());

							self.image_info = Some(info);
						}
					} else {
						ui.label("Couldn't read file");
					}
				}
			});
		}
	}

	fn create_sidebar(&mut self, ctx: &egui::Context) {
		egui::SidePanel::left("side_panel").show(ctx, |ui| {
			ui.heading("DMI Metadata Tool");

			// Show dropped files (if any):
			self.load_file_or_err(ui);

			// Clean the dropped files list as soon as we have an image. Needed to reload a new, future image.
			if self.img.is_some() {
				self.dropped_files.clear();
			}

			if self.img.is_some() {
				if let Some(image_info) = &self.image_info {
					ui.horizontal(|ui| {
						ui.label("Loaded file:");
						ui.monospace(&image_info.name);
					});
				} else {
					ui.label("Error: No File Info");
				}

				if ui.button("Extract Metadata").clicked() {}

				if ui.button("Apply Metadata").clicked() {}
			}

			ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
				ui.horizontal(|ui| {
					ui.spacing_mut().item_spacing.x = 0.0;
					ui.label("Made by ZeWaka (");
					ui.hyperlink_to("GitHub", "https://github.com/ZeWaka/dmi-meta-tool");
					ui.label(")");
				});
			});
		});
	}

	fn create_image_preview(&mut self, ctx: &egui::Context) {
		egui::CentralPanel::default().show(ctx, |ui| {

            let image_height = ui.available_height() * 0.70; // image takes up 70% of the height at max
            ui.allocate_ui_with_layout( vec2(ui.available_width(), image_height), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                // Get available space for the image
                self.img_offset = ui.cursor().left_top();
                self.available_height = ui.available_height();
                self.available_width = ui.available_width();

                match &self.img {
                    Some(i) => ui.image(i.texture_id(ctx), i.size_vec2()), // Preview
                    _ => {
                        ui.centered_and_justified(|ui| {
                            ui.heading(RichText::new("Drop file here").strong())
                        })
                        .response
                    } // No image
                };
                ui.add(egui::Separator::default().grow(8.0));
                ui.label("Sed tincidunt enim non velit pharetra, id viverra risus pretium. Mauris eu risus finibus, placerat dolor et, condimentum sapien. Sed tincidunt enim non velit pharetra, id viverra risus pretium. Mauris eu risus finibus, placerat dolor et, condimentum sapien. Sed tincidunt enim non velit pharetra, id viverra risus pretium. Mauris eu risus finibus, placerat dolor et, condimentum sapien. Sed tincidunt enim non velit pharetra, id viverra risus pretium. Mauris eu risus finibus, placerat dolor et, condimentum sapien. Sed tincidunt enim non velit pharetra, id viverra risus pretium. Mauris eu risus finibus, placerat dolor et, condimentum sapien. Sed tincidunt enim non velit pharetra, id viverra risus pretium. Mauris eu risus finibus, placerat dolor et, condimentum sapien. Sed tincidunt enim non velit pharetra, id viverra risus pretium. Mauris eu risus finibus, placerat dolor et, condimentum sapien. Sed tincidunt enim non velit pharetra, id viverra risus pretium. Mauris eu risus finibus, placerat dolor et, condimentum sapien. Sed tincidunt enim non velit pharetra, id viverra risus pretium. Mauris eu risus finibus, placerat dolor et, condimentum sapien. Sed tincidunt enim non velit pharetra, id viverra risus pretium. Mauris eu risus finibus, placerat dolor et, condimentum sapien. Sed tincidunt enim non velit pharetra, id viverra risus pretium. Mauris eu risus finibus, placerat dolor et, condimentum sapien. Sed tincidunt enim non velit pharetra, id viverra risus pretium. Mauris eu risus finibus, placerat dolor et, condimentum sapien.");
            });
        });
	}
}

impl eframe::App for MetadataApp {
	/// Called each time the UI needs repainting, which may be many times per second.
	/// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		// For inspiration and more examples, go to https://emilk.github.io/egui

		#[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
		egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
			// The top panel is often a good place for a menu bar:
			egui::menu::bar(ui, |ui| {
				ui.menu_button("File", |ui| {
					if ui.button("Quit").clicked() {
						_frame.close();
					}
				});
			});
		});

		self.create_sidebar(ctx);
		self.create_image_preview(ctx);

		self.preview_files_being_dropped(ctx);

		ctx.input(|i| {
			if !i.raw.dropped_files.is_empty() {
				self.dropped_files = i.raw.dropped_files.clone();
			}
		});
	}
}

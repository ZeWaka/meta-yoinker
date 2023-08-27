use crate::dmi_window::{
	create_image_preview, create_meta_viewer, FileInfo, ImageMetadata, UIWindow,
};
use egui::{mutex::Mutex, Align2, DroppedFile, FontFamily, FontId, RichText, TextStyle};
use egui_extras::RetainedImage;
use once_cell::sync::Lazy;
use std::{cell::RefCell, io::Cursor, rc::Rc};

#[derive(Default)]
pub struct MetadataTool {
	windows: Vec<UIWindow>,
	dropped_files: Vec<egui::DroppedFile>,
	pub toasts: egui_toast::Toasts,
}

pub static GLOB_COPIED_METADATA: Lazy<Mutex<Option<CopiedMetadata>>> =
	Lazy::new(|| Mutex::new(None));

#[derive(Default)]
pub struct CopiedMetadata {
	pub orig_file: String,
	pub metadata: dmi::ztxt::RawZtxtChunk,
}

#[derive(Clone, Default)]
pub enum MetadataStatus {
	#[default]
	NoMeta,
	Meta(Rc<RefCell<String>>),
}

fn configure_text_styles(ctx: &egui::Context) {
	use FontFamily::{Monospace, Proportional};

	let mut style = (*ctx.style()).clone();
	style.text_styles = [
		(TextStyle::Heading, FontId::new(25.0, Proportional)),
		(TextStyle::Body, FontId::new(16.0, Proportional)),
		(TextStyle::Monospace, FontId::new(12.0, Monospace)),
		(TextStyle::Button, FontId::new(12.0, Proportional)),
		(TextStyle::Small, FontId::new(8.0, Proportional)),
	]
	.into();
	ctx.set_style(style);
}

impl MetadataTool {
	/// Called once before the first frame.
	#[must_use]
	pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
		// This is also where you can customize the look and feel of egui using
		// `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
		configure_text_styles(&cc.egui_ctx);

		let toasts = egui_toast::Toasts::new()
			.anchor(Align2::RIGHT_BOTTOM, (-10.0, -10.0)) // 10 units from the bottom right corner
			.direction(egui::Direction::BottomUp);

		Self {
			windows: Vec::new(),
			dropped_files: Vec::new(),
			toasts,
		}
	}

	fn preview_files_being_dropped(ctx: &egui::Context) {
		use egui::{Color32, Id, LayerId, Order};

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
					text += "\nImage";
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

	/// Loads the contents of a DroppedFile, depending on platform. Returns the bytes.
	fn load_file_contents(file: &DroppedFile) -> Option<Vec<u8>> {
		if let Some(path) = &file.path {
			// Load file contents from the path (non-web)
			if let Ok(mut file) = std::fs::File::open(path) {
				let mut buffer = Vec::new();
				if std::io::Read::read_to_end(&mut file, &mut buffer).is_ok() {
					return Some(buffer);
				}
			}
		} else if let Some(bytes) = &file.bytes {
			// Use existing bytes (web)
			return Some(bytes.clone().to_vec());
		}
		None
	}

	fn load_files_or_err(&mut self, ui: &mut egui::Ui) {
		if !self.dropped_files.is_empty() {
			ui.group(|ui| {
				ui.label("Dropped files:");

				for file in &self.dropped_files {
					if let Some(bytes) = Self::load_file_contents(file) {
						if bytes.is_empty() {
							return;
						}

						let mut buffer: Vec<u8> = Vec::new();
						let mut writer = Cursor::new(&mut buffer);
						let bytes_reader = Cursor::new(&bytes);

						let mut i = match image::load_from_memory_with_format(
							&bytes,
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

						if let Ok(raw_dmi) = dmi::RawDmi::load(bytes_reader) {
							let new_mwin = UIWindow {
								id: uuid::Uuid::new_v4(),
								img: {
									let h = (ui.available_height() * 2.0) as u32;
									let w = (ui.available_width() * 2.0) as u32;

									i = i.resize(w, h, image::imageops::FilterType::Nearest);
									i.write_to(&mut writer, image::ImageFormat::Png).unwrap();

									Some(Rc::new(
										RetainedImage::from_image_bytes("img", &buffer).unwrap(),
									))
								},
								metadata: Rc::new({
									ImageMetadata {
										img_metadata_raw: { raw_dmi.chunk_ztxt.clone() },
										img_metadata_text: {
											raw_dmi.chunk_ztxt.map_or(
												MetadataStatus::NoMeta,
												|metadata| {
													MetadataStatus::Meta(Rc::new(RefCell::new(
														format!("{:#?}", metadata),
													)))
												},
											)
										},
										image_info: {
											let name_str: String;
											if let Some(path) = &file.path {
												if let Some(file_name_osstr) = path.file_name() {
													name_str = file_name_osstr
														.to_string_lossy()
														.into_owned();
												} else {
													name_str = "???".to_owned();
												}
											} else if !file.name.is_empty() {
												name_str = file.name.clone();
											} else {
												name_str = "???".to_owned();
											};
											FileInfo { name: name_str }
										},
									}
								}),
								is_open: true,
							};
							self.windows.push(new_mwin);
						}
					}
				}
			});
		}
	}

	fn create_sidebar(&mut self, ctx: &egui::Context) {
		egui::SidePanel::left("side_panel").show(ctx, |ui| {
			ui.heading("MetaYoinker 😈");

			// Show dropped files (if any):
			self.load_files_or_err(ui);

			if self.windows.is_empty() {
				ui.label("No Files Loaded");
			} else {
				ui.label(
					(if self.windows.len() > 1 {
						"Loaded files:"
					} else {
						"Loaded file:"
					})
					.to_string(),
				);
				// Clean the dropped files list as soon as we have an image. Needed to reload a new, future image.
				self.dropped_files.clear();
				for window in &self.windows {
					if window.img.is_some() {
						ui.monospace(&window.metadata.image_info.name);
					}
				}
			}

			ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
				ui.label("Made by ZeWaka");
				ui.hyperlink_to("GitHub", env!("CARGO_PKG_REPOSITORY"));
			});
		});
	}
}

impl eframe::App for MetadataTool {
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

		for mwindow in &self.windows {
			if mwindow.is_open {
				egui::Window::new(&mwindow.metadata.image_info.name)
					.id(mwindow.id.to_string().into())
					.show(ctx, |ui| {
						create_meta_viewer(mwindow, ui, &mwindow.metadata, &mut self.toasts);
						create_image_preview(mwindow, ui, ctx);
					});
			}
		}

		Self::preview_files_being_dropped(ctx);

		self.create_sidebar(ctx);

		egui::CentralPanel::default().show(ctx, |ui| {
			ui.centered_and_justified(|ui| {
				ui.heading(RichText::new("Drag & drop file(s) here").strong())
			})
		});

		ctx.input(|i| {
			if !i.raw.dropped_files.is_empty() {
				self.dropped_files = i.raw.dropped_files.clone();
			}
		});

		self.toasts.show(ctx)
	}
}

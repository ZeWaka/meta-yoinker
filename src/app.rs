use egui::{vec2, DroppedFile, FontFamily, FontId, RichText, TextStyle};
use egui_extras::RetainedImage;
use std::{cell::RefCell, io::Cursor, rc::Rc};

#[derive(Default)]
pub struct MetadataTool {
	windows: Vec<UIWindow>,
	dropped_files: Vec<egui::DroppedFile>,
}

#[derive(Default)]
struct UIWindow {
	id: uuid::Uuid,
	img: Option<Rc<RetainedImage>>,
	metadata: Rc<ImageMetadata>,
}

#[derive(Clone, Default)]
struct ImageMetadata {
	img_metadata_raw: Option<dmi::ztxt::RawZtxtChunk>,
	img_metadata_text: MetadataStatus,
	image_info: Option<FileInfo>,
}

#[derive(Clone, Default)]
struct FileInfo {
	name: String,
	path: String,
}

#[derive(Clone, PartialEq)]
enum MetadataStatus {
	NotLoaded,
	NoMeta,
	Meta(Rc<RefCell<String>>),
}

impl Default for MetadataStatus {
	fn default() -> Self {
		MetadataStatus::NotLoaded
	}
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
		Self::default()
	}

	fn preview_files_being_dropped(ctx: &egui::Context) {
		use egui::{Align2, Color32, Id, LayerId, Order};

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
									let h = 100; //(new_mwin.available_height * 1.0) as u32;
									let w = 100; //(new_mwin.available_width * 1.0) as u32;

									i = i.resize(w, h, image::imageops::FilterType::Nearest);
									i.write_to(&mut writer, image::ImageFormat::Png).unwrap();

									Some(Rc::new(
										RetainedImage::from_image_bytes("img", &buffer).unwrap(),
									))
								},
								metadata: Rc::new({
									ImageMetadata {
										img_metadata_raw: {
											if let Some(metadata) = raw_dmi.chunk_ztxt.clone() {
												Some(metadata)
											} else {
												None
											}
										},
										img_metadata_text: {
											if let Some(metadata) = raw_dmi.chunk_ztxt {
												MetadataStatus::Meta(Rc::new(RefCell::new(
													format!("{:#?}", metadata),
												)))
											} else {
												MetadataStatus::NoMeta
											}
										},
										image_info: Some({
											let name_str: String;
											let mut path_str: String = String::new();
											if let Some(path) = &file.path {
												path_str = path.display().to_string();
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
											FileInfo {
												name: name_str,
												path: path_str,
											}
										}),
									}
								}),
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
				for window in &self.windows {
					if window.img.is_some() {
						// Clean the dropped files list as soon as we have an image. Needed to reload a new, future image.
						self.dropped_files.clear();

						if let Some(image_info) = &window.metadata.image_info {
							ui.label("Loaded file:");
							ui.monospace(&image_info.name);
						} else {
							ui.label("Error: No File Info");
						}
					}
				}
			}

			ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
				ui.horizontal(|ui| {
					ui.spacing_mut().item_spacing.x = 0.0;
					ui.label("Made by ZeWaka (");
					ui.hyperlink_to("GitHub", env!("CARGO_PKG_REPOSITORY"));
					ui.label(")");
				});
			});
		});
	}

	fn create_image_preview(
		&mut self,
		ctx: &egui::Context,
		img: &Option<Rc<RetainedImage>>,
		metadata: &Rc<ImageMetadata>,
	) {
		self.create_meta_viewer(ctx, metadata);
		egui::CentralPanel::default().show(ctx, |ui| {
			let image_height = ui.available_height() * 1.0; // image takes up 70% of the height at max
			ui.allocate_ui_with_layout(
				vec2(ui.available_width(), image_height),
				egui::Layout::top_down(egui::Align::Center),
				|ui| {
					match img {
						Some(i) => ui.image(i.texture_id(ctx), i.size_vec2()), // Preview
						_ => {
							ui.centered_and_justified(|ui| {
								ui.heading(RichText::new("Drop file here").strong())
							})
							.response
						} // No image
					};
				},
			);
		});
	}
	fn create_meta_viewer(&mut self, ctx: &egui::Context, metadata: &Rc<ImageMetadata>) {
		egui::TopBottomPanel::bottom("gaming").show(ctx, |ui| {
			ui.allocate_ui_with_layout(
				vec2(ui.available_width(), ui.available_height() * 0.8),
				egui::Layout::left_to_right(egui::Align::Center),
				|ui| {
					// Center the content horizontally
					match &metadata.img_metadata_text {
						MetadataStatus::Meta(metadata) => {
							let cloned_metadata = metadata.clone();
							ui.code_editor(&mut cloned_metadata.as_ref().borrow().as_str());
						}
						MetadataStatus::NoMeta => {
							ui.code_editor(&mut String::from("No Metadata"));
						}
						MetadataStatus::NotLoaded => {
							ui.code_editor(&mut String::from("Nothing Loaded"));
						}
					}
					if ui.button(RichText::new("➡").size(20.0)).clicked() {
						// TODO: copy data
					}
					//TODO: Save and display data
					ui.code_editor(&mut String::from("Nothing Saved"));
				},
			);
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
			egui::Window::new("Main thread").show(ctx, |ui| {
				ui.label("Hello World!");
				self.create_image_preview(ctx, &mwindow.img, &mwindow.metadata);
			});
		}

		Self::preview_files_being_dropped(ctx);

		self.create_sidebar(ctx);

		ctx.input(|i| {
			if !i.raw.dropped_files.is_empty() {
				self.dropped_files = i.raw.dropped_files.clone();
			}
		});
	}
}

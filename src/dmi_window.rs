use crate::app::{CopiedMetadata, MetadataStatus, GLOB_COPIED_METADATA};
use egui::{text::LayoutJob, vec2, RichText, TextFormat};
use egui_extras::RetainedImage;
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};
use std::rc::Rc;

pub struct UIWindow {
	pub id: uuid::Uuid,
	pub img: Option<Rc<RetainedImage>>,
	pub metadata: Rc<ImageMetadata>,
	pub is_open: bool,
}

pub struct ImageMetadata {
	pub img_metadata_raw: Option<dmi::ztxt::RawZtxtChunk>,
	pub img_metadata_text: MetadataStatus,
	pub image_info: FileInfo,
}

pub struct FileInfo {
	pub name: String,
}

pub fn create_image_preview(mwindow: &UIWindow, ui: &mut egui::Ui, ctx: &egui::Context) {
	egui::TopBottomPanel::top(format!("{}_img", mwindow.id)).show_inside(ui, |ui| {
		ui.allocate_ui_with_layout(
			vec2(ui.available_width(), ui.available_height()),
			egui::Layout::top_down(egui::Align::Center),
			|ui| {
				mwindow.img.as_ref().map_or_else(
					|| unreachable!(),
					|i| ui.image(i.texture_id(ctx), i.size_vec2()),
				);
			},
		);
	});
}

pub fn create_meta_viewer(
	mwindow: &UIWindow,
	ui: &mut egui::Ui,
	metadata: &Rc<ImageMetadata>,
	toasts: &mut Toasts,
) {
	egui::TopBottomPanel::bottom(format!("{}_meta", mwindow.id)).show_inside(ui, |ui| {
		ui.allocate_ui_with_layout(
			vec2(ui.available_width(), ui.available_height()),
			egui::Layout::left_to_right(egui::Align::Center),
			|ui| {
				if ui.button(RichText::new("Copy").size(20.0)).clicked() {
					if let Some(raw_meta) = &metadata.img_metadata_raw {
						let new_meta = {
							Some(CopiedMetadata {
								orig_file: metadata.image_info.name.clone(),
								metadata: raw_meta.clone(),
							})
						};
						*GLOB_COPIED_METADATA.lock() = new_meta;
						toasts.add(Toast {
							text: format!("Copied metadata for {}", metadata.image_info.name)
								.into(),
							kind: ToastKind::Success,
							options: ToastOptions::default()
								.duration_in_seconds(2.0)
								.show_progress(true),
						});
					}
				}
				let mut metadata_text = LayoutJob::default();
				metadata_text.append("Metadata:", 0.0, TextFormat::default());
				if metadata.img_metadata_raw.is_some() {
					metadata_text.append(
						"Yes",
						1.0,
						TextFormat {
							color: egui::Color32::LIGHT_GREEN,
							..TextFormat::default()
						},
					);
				} else {
					metadata_text.append(
						"None",
						1.0,
						TextFormat {
							color: egui::Color32::LIGHT_RED,
							..TextFormat::default()
						},
					);
				};
				egui::CollapsingHeader::new(metadata_text).show(ui, |ui| {
					egui::ScrollArea::vertical().show(ui, |ui| match &metadata.img_metadata_text {
						MetadataStatus::Meta(metadata) => {
							let cloned_metadata = metadata.clone();
							ui.code_editor(&mut cloned_metadata.as_ref().borrow().as_str());
						}
						MetadataStatus::NoMeta => {
							ui.label("No Metadata!");
						}
					});
				});
			},
		);
	});
}

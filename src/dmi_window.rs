use crate::app::MetadataStatus;
use egui::{vec2, RichText};
use egui_extras::RetainedImage;
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

pub fn create_meta_viewer(mwindow: &UIWindow, ui: &mut egui::Ui, metadata: &Rc<ImageMetadata>) {
	egui::TopBottomPanel::bottom(format!("{}_meta", mwindow.id)).show_inside(ui, |ui| {
		ui.allocate_ui_with_layout(
			vec2(ui.available_width(), ui.available_height()),
			egui::Layout::left_to_right(egui::Align::Center),
			|ui| {
				egui::CollapsingHeader::new("Metadata").show(ui, |ui| {
					if ui.button(RichText::new("Copy").size(20.0)).clicked() {
						// TODO: copy data
					}
					match &metadata.img_metadata_text {
						MetadataStatus::Meta(metadata) => {
							let cloned_metadata = metadata.clone();
							ui.code_editor(&mut cloned_metadata.as_ref().borrow().as_str());
						}
						MetadataStatus::NoMeta => {
							ui.code_editor(&mut String::from("No Metadata"));
						}
						MetadataStatus::NotLoaded => {
							ui.code_editor(&mut String::from("Error: Nothing Loaded"));
						}
					}
				});
			},
		);
	});
}

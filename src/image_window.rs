use crate::{app::GLOB_COPIED_METADATA, metadata::ImageMetadata};
use egui::{text::LayoutJob, vec2, RichText, TextFormat};
use egui_extras::RetainedImage;
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};
use poll_promise::Promise;
use std::{cell::RefCell, rc::Rc};

pub struct ImageWindow {
	pub id: egui::Id,
	pub img: Rc<RetainedImage>,
	pub dmi: dmi::RawDmi,
	pub metadata: Rc<ImageMetadata>,
	pub is_open: RefCell<bool>,
}

pub fn create_image_preview(mwindow: &ImageWindow, ui: &mut egui::Ui, ctx: &egui::Context) {
	egui::CentralPanel::default().show_inside(ui, |ui| {
		let img = mwindow.img.as_ref();
		ui.image(img.texture_id(ctx), img.size_vec2());
	});
}

pub fn create_meta_viewer(
	img_win: &ImageWindow,
	ui: &mut egui::Ui,
	metadata: &Rc<ImageMetadata>,
	toasts: &RefCell<&mut Toasts>,
) {
	egui::TopBottomPanel::bottom(format!("{:?}_meta", img_win.id)).show_inside(ui, |ui| {
		ui.add_space(6.0);
		ui.allocate_ui_with_layout(
			vec2(ui.available_width(), ui.available_height()),
			egui::Layout::left_to_right(egui::Align::Center),
			|ui| {
				ui.add_enabled_ui(metadata.ztxt_meta.is_some(), |ui| {
					if ui
						.button(RichText::new(egui_phosphor::regular::COPY).size(25.0))
						.on_hover_text("Copy")
						.clicked()
					{
						copy_metadata(metadata, toasts);
					}
				});

				let clipboard_meta_avail = GLOB_COPIED_METADATA.lock().is_some();
				ui.add_enabled_ui(clipboard_meta_avail, |ui| {
					if ui
						.button(
							RichText::new(format!(
								"{}{}",
								egui_phosphor::regular::CLIPBOARD_TEXT,
								egui_phosphor::regular::DOWNLOAD
							))
							.size(25.0),
						)
						.on_hover_text(if metadata.ztxt_meta.is_some() {
							"Overwrite & Download"
						} else {
							"Paste & Download"
						})
						.clicked()
					{
						paste_metadata(
							img_win,
							toasts,
							GLOB_COPIED_METADATA.lock().clone().unwrap(),
						);
					}
				});

				let mut metadata_text = LayoutJob::default();
				metadata_text.append("Metadata:", 0.0, TextFormat::default());
				if metadata.ztxt_meta.is_some() {
					metadata_text.append(
						"Yes",
						2.0,
						TextFormat {
							color: egui::Color32::LIGHT_GREEN,
							..TextFormat::default()
						},
					);
				} else {
					metadata_text.append(
						"None",
						2.0,
						TextFormat {
							color: egui::Color32::LIGHT_RED,
							..TextFormat::default()
						},
					);
				};
				egui::CollapsingHeader::new(metadata_text).show(ui, |ui| {
					egui::ScrollArea::vertical().show(ui, |ui| {
						ui.code_editor(&mut format!("{}", metadata));
					});
				});
			},
		);
	});
}

fn copy_metadata(metadata: &Rc<ImageMetadata>, toasts: &RefCell<&mut Toasts>) {
	if let Some(raw_meta) = &metadata.ztxt_meta {
		let new_meta = {
			Some(ImageMetadata {
				file_name: metadata.file_name.clone(),
				ztxt_meta: Some(raw_meta.clone()),
			})
		};
		*GLOB_COPIED_METADATA.lock() = new_meta;
		let mut toast_lock = toasts.borrow_mut();
		toast_lock.add(Toast {
			text: format!("Copied metadata for {}", metadata.file_name).into(),
			kind: ToastKind::Success,
			options: ToastOptions::default()
				.duration_in_seconds(1.5)
				.show_progress(true),
		});
	}
}

fn paste_metadata(mwindow: &ImageWindow, toasts: &RefCell<&mut Toasts>, metadata: ImageMetadata) {
	let mut filename = mwindow.metadata.file_name.clone();
	// pls forgive me for this thx
	filename = filename[..filename.len() - 4].to_owned() + ".dmi";

	let mut new_dmi = mwindow.dmi.clone();
	new_dmi.chunk_ztxt = metadata.ztxt_meta;

	// Create buffer for the dmi to save its binary data to
	let mut buffer = Vec::with_capacity(std::mem::size_of_val(&new_dmi));
	new_dmi.save(&mut buffer).unwrap();

	#[allow(unused_variables)] // Not used on web
	let promise = Promise::spawn_local(async move {
		if let Some(path) = rfd::AsyncFileDialog::new()
			.add_filter("DM Image File", &["dmi"])
			.set_title("Download DMI")
			.set_file_name(filename)
			.set_directory("/")
			.save_file()
			.await
		{
			path.write(buffer.as_slice()).await.unwrap();
		}
	});

	// We do not want to block on wasm, since we're running in an async context already
	#[cfg(not(target_arch = "wasm32"))]
	promise.block_and_take();

	let mut toast_lock = toasts.borrow_mut();
	toast_lock.add(Toast {
		text: format!("Downloaded {}", mwindow.metadata.file_name).into(),
		kind: ToastKind::Success,
		options: ToastOptions::default()
			.duration_in_seconds(1.5)
			.show_progress(true),
	});
}

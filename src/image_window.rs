use crate::{app::GLOB_COPIED_METADATA, metadata::ImageMetadata};
use egui::{text::LayoutJob, vec2, RichText, TextFormat};
use egui_extras::RetainedImage;
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};
use std::{cell::RefCell, rc::Rc};

pub struct ImageWindow {
	pub id: uuid::Uuid,
	pub img: Option<Rc<RetainedImage>>,
	pub dmi: dmi::RawDmi,
	pub metadata: Rc<ImageMetadata>,
	pub is_open: RefCell<bool>,
}

pub fn create_image_preview(mwindow: &ImageWindow, ui: &mut egui::Ui, ctx: &egui::Context) {
	egui::TopBottomPanel::top(format!("{}_img", mwindow.id)).show_inside(ui, |ui| {
		ui.allocate_ui_with_layout(
			vec2(ui.available_width(), ui.available_height()),
			egui::Layout::top_down(egui::Align::Center),
			|ui| {
				mwindow.img.as_ref().map_or_else(
					|| unreachable!(),
					|i| ui.image(i.texture_id(ctx), i.size_vec2() * 0.9), // display image w/ margin
				);
			},
		);
	});
}

pub fn create_meta_viewer(
	img_win: &ImageWindow,
	ui: &mut egui::Ui,
	metadata: &Rc<ImageMetadata>,
	toasts: &RefCell<&mut Toasts>,
) {
	egui::TopBottomPanel::bottom(format!("{}_meta", img_win.id)).show_inside(ui, |ui| {
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

				let has_metadata = GLOB_COPIED_METADATA.lock().is_some();
				ui.add_enabled_ui(has_metadata, |ui| {
					if ui
						.button(
							RichText::new(format!(
								"{} {}",
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
						paste_metadata(img_win, toasts);
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

#[cfg(not(target_arch = "wasm32"))]
fn paste_metadata(mwindow: &ImageWindow, toasts: &RefCell<&mut Toasts>) {
	let meta_to_paste = GLOB_COPIED_METADATA.lock().clone();

	if let Some(mut path) = rfd::FileDialog::new()
		.add_filter("DM Image Files", &["dmi"])
		.set_title("Download File")
		.set_file_name(mwindow.metadata.file_name.as_str())
		.set_directory("/")
		.save_file()
	{
		path.set_extension("dmi");
		let mut old_dmi = mwindow.dmi.clone();

		old_dmi.chunk_ztxt = meta_to_paste.unwrap().ztxt_meta;

		let mut file = std::fs::File::create(path).unwrap();
		old_dmi.save(&mut file).unwrap();
	}

	let mut toast_lock = toasts.borrow_mut();
	toast_lock.add(Toast {
		text: format!("Downloaded {}", mwindow.metadata.file_name).into(),
		kind: ToastKind::Success,
		options: ToastOptions::default()
			.duration_in_seconds(1.5)
			.show_progress(true),
	});
}

#[cfg(target_arch = "wasm32")]
fn paste_metadata(metadata: &Rc<ImageMetadata>, toasts: &RefCell<&mut Toasts>) {
	let meta_to_paste = GLOB_COPIED_METADATA.lock().clone();

	let promise = js_sys::Promise::new(&mut move |res, _rej| {
		let file_reader = web_sys::FileReader::new().unwrap();

		let fr = file_reader.clone();
		let closure = Closure::wrap(Box::new(move || {
			res.call1(&JsValue::undefined(), &fr.result().unwrap())
				.unwrap();
		}) as Box<dyn FnMut()>);

		file_reader.set_onload(Some(closure.as_ref().unchecked_ref()));

		closure.forget();

		file_reader.read_as_array_buffer(&self.0).unwrap();
	});

	let future = wasm_bindgen_futures::JsFuture::from(promise);

	let res = future.await.unwrap();

	let mut toast_lock = toasts.borrow_mut();
	toast_lock.add(Toast {
		text: format!("Downloaded {}", metadata.file_name).into(),
		kind: ToastKind::Success,
		options: ToastOptions::default()
			.duration_in_seconds(1.5)
			.show_progress(true),
	});
}

use crate::{app::GLOB_COPIED_METADATA, MetadataTool};

use egui::{Margin, RichText, Rounding, Stroke};
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};

pub fn create_sidebar(app: &mut MetadataTool, ctx: &egui::Context) {
	egui::SidePanel::left("side_panel").show(ctx, |ui| {
		ui.heading("MetaYoinker 😈");

		// Show dropped files (if any):
		app.load_files_or_err(ui);

		if app.windows.is_empty() {
			ui.label("No Files Loaded");
		} else {
			ui.label(
				(if app.windows.len() > 1 {
					"Loaded files:"
				} else {
					"Loaded file:"
				})
				.to_string(),
			);
			// Clean the dropped files list as soon as we have an image. Needed to reload a new, future image.
			app.dropped_files.clear();
			for window in &app.windows {
				if window.img.is_some() {
					ui.monospace(&window.metadata.image_info.name);
				}
			}
		}

		ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
			ui.label("Made by ZeWaka");
			ui.horizontal(|ui| {
				ui.hyperlink_to("GitHub", env!("CARGO_PKG_REPOSITORY"));
				egui::global_dark_light_mode_buttons(ui);
			});

			ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
				egui::Frame::default()
					.stroke(Stroke {
						width: 1.0,
						color: {
							let meta_guard = GLOB_COPIED_METADATA.lock();
							if (*meta_guard).is_some() {
								egui::Color32::LIGHT_GREEN
							} else {
								egui::Color32::LIGHT_RED
							}
						},
					})
					.rounding(Rounding::same(2.0))
					.inner_margin(Margin::same(6.0))
					.show(ui, |ui| {
						let meta_g = GLOB_COPIED_METADATA.lock();
						let meta_guard = &*meta_g;
						if let Some(meta) = meta_guard {
							ui.label(meta.orig_file.clone());
						} else {
							ui.label(RichText::new("None").color(egui::Color32::LIGHT_RED));
						}
						ui.add_space(5.0);
						let has_meta_in_clipboard = meta_guard.is_some();
						drop(meta_g); // Release the lock
						ui.horizontal(|ui| {
							ui.heading("Clipboard:");
							ui.add_enabled_ui(has_meta_in_clipboard, |ui| {
								if ui.button(RichText::new("Clear").size(20.0)).clicked() {
									clear_meta_clipboard(&mut app.toasts);
								}
							});
						});
					});
			});
		});
	});
}

fn clear_meta_clipboard(toasts: &mut Toasts) {
	toasts.add(Toast {
		text: "Cleared clipboard".into(),
		kind: ToastKind::Success,
		options: ToastOptions::default()
			.duration_in_seconds(2.0)
			.show_progress(true),
	});
	*GLOB_COPIED_METADATA.lock() = None;
}

use dmi::ztxt::RawZtxtChunk;
use egui::DroppedFile;
use std::fmt::Display;

#[derive(Default, Clone)]
pub struct ImageMetadata {
	pub ztxt_meta: Option<RawZtxtChunk>,
	pub file_name: String,
}

impl ImageMetadata {
	pub fn new(ztxt: Option<RawZtxtChunk>, file: &DroppedFile) -> Self {
		Self {
			ztxt_meta: { ztxt },
			file_name: { Self::get_file_name(file) },
		}
	}

	fn get_file_name(file: &DroppedFile) -> String {
		file.path.as_ref().map_or_else(
			|| {
				// Web file paths
				if !file.name.is_empty() {
					file.name.clone()
				} else {
					"???".to_owned()
				}
			},
			|path| {
				// OS file paths
				path.file_name().map_or_else(
					|| "???".to_owned(),
					|fname_osstr| fname_osstr.to_string_lossy().into_owned(),
				)
			},
		)
	}
}

impl Display for ImageMetadata {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let meta_str = self
			.ztxt_meta
			.as_ref()
			.map(|metadata| format!("{:#?}", metadata));
		write!(
			f,
			"# {}\n{}",
			self.file_name,
			meta_str.unwrap_or_else(|| "No metadata".to_owned())
		)
	}
}

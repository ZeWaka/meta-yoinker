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
			file_name: {
				let name_str: String;
				if let Some(path) = &file.path {
					// Handle Native OS file paths
					if let Some(file_name_osstr) = path.file_name() {
						name_str = file_name_osstr.to_string_lossy().into_owned();
					} else {
						name_str = "???".to_owned();
					}
				} else if !file.name.is_empty() {
					// Web file paths
					name_str = file.name.clone();
				} else {
					name_str = "???".to_owned();
				};
				name_str
			},
		}
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

use std::fmt::Display;

use egui::DroppedFile;

pub struct ImageMetadata {
	pub img_metadata_raw: Option<dmi::ztxt::RawZtxtChunk>,
	pub file_name: String,
}

impl Display for ImageMetadata {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let meta_str = self
			.img_metadata_raw
			.as_ref()
			.map(|metadata| format!("{:#?}", metadata));
		write!(
			f,
			"{}",
			meta_str.unwrap_or_else(|| "No metadata".to_owned())
		)
	}
}

#[derive(Default)]
pub struct CopiedMetadata {
	pub file_name: String,
	pub metadata: dmi::ztxt::RawZtxtChunk,
}

pub fn extract_metadata(raw_dmi: dmi::RawDmi, file: &DroppedFile) -> ImageMetadata {
	let ztxt_metadata = raw_dmi.chunk_ztxt;
	ImageMetadata {
		img_metadata_raw: { ztxt_metadata },
		file_name: {
			let name_str: String;
			if let Some(path) = &file.path {
				if let Some(file_name_osstr) = path.file_name() {
					name_str = file_name_osstr.to_string_lossy().into_owned();
				} else {
					name_str = "???".to_owned();
				}
			} else if !file.name.is_empty() {
				name_str = file.name.clone();
			} else {
				name_str = "???".to_owned();
			};
			name_str
		},
	}
}

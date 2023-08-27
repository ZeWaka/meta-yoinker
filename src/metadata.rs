use egui::DroppedFile;

pub struct ImageMetadata {
	pub img_metadata_raw: Option<dmi::ztxt::RawZtxtChunk>,
	pub img_metadata_text: Option<String>,
	pub image_info: FileInfo,
}

pub struct FileInfo {
	pub name: String,
}

pub fn extract_metadata(raw_dmi: dmi::RawDmi, file: &DroppedFile) -> ImageMetadata {
	ImageMetadata {
		img_metadata_raw: { raw_dmi.chunk_ztxt.clone() },
		img_metadata_text: {
			raw_dmi
				.chunk_ztxt
				.map(|metadata| format!("{:#?}", metadata))
		},
		image_info: {
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
			FileInfo { name: name_str }
		},
	}
}

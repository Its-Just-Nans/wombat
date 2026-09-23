//! Document

use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use bladvak::utils::document::DocumentTrait;
use file_format::FileFormat;

use crate::offset::Offset;
use crate::panels::FileInfoData;
use crate::selection::Selection;
use crate::windows::WindowsData;

/// Document
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub(crate) struct Document {
    /// Binary file data
    #[serde(skip)]
    pub(crate) binary_file: Arc<Vec<u8>>,
    /// Filename of the file
    #[serde(skip)]
    pub(crate) filename: PathBuf,
    /// Selection
    pub(crate) selection: Selection,
    /// Scroll area offset
    pub(crate) offset: Offset,
    /// File info
    pub(crate) file_format: FileInfoData,
    /// Bytes per line
    pub(crate) bytes_per_line: usize,
    /// Windows
    pub(crate) windows_data: WindowsData,
}

impl Default for Document {
    fn default() -> Self {
        let (data, path) = Self::load_default_file();
        let file_fmt = FileFormat::from_bytes(&data);
        let file_format = FileInfoData {
            file_type: file_fmt.media_type().to_string(),
            extension: file_fmt.extension().to_string(),
            name: file_fmt.name().to_string(),
        };
        Self {
            binary_file: Arc::new(data),
            filename: path,
            file_format,
            selection: Selection::default(),
            offset: Offset::default(),
            bytes_per_line: 32,
            windows_data: WindowsData::new(),
        }
    }
}

/// default file (wombat icon)
const LOGO_ASSET: &[u8] = include_bytes!("../assets/icon-1024.png");

impl Document {
    /// Load default file
    pub(crate) fn load_default_file() -> (Vec<u8>, PathBuf) {
        (LOGO_ASSET.to_vec(), PathBuf::from("wombat.png"))
    }

    /// create a new document
    pub(crate) fn new(bytes: Vec<u8>, filename: PathBuf) -> Self {
        let file_fmt = FileFormat::from_bytes(&bytes);
        let file_format = FileInfoData {
            file_type: file_fmt.media_type().to_string(),
            extension: file_fmt.extension().to_string(),
            name: file_fmt.name().to_string(),
        };
        Self {
            binary_file: Arc::new(bytes),
            filename,
            file_format,
            selection: Selection::default(),
            offset: Offset::default(),
            bytes_per_line: 32,
            windows_data: WindowsData::new(),
        }
    }

    /// Get file info - load if needed
    pub(crate) fn get_file_format(&mut self) -> &FileInfoData {
        &self.file_format
    }
    /// Go to the selected range
    pub(crate) fn go_to_range(&mut self, range: RangeInclusive<usize>) {
        let start = *range.start();
        self.selection.range = Some((start, *range.end()));
        self.offset.go_to_index(start, self.bytes_per_line);
    }

    /// Mark document as stale
    pub(crate) fn stale(&mut self) {
        let file_fmt = FileFormat::from_bytes(&*self.binary_file);
        self.file_format = FileInfoData {
            file_type: file_fmt.media_type().to_string(),
            extension: file_fmt.extension().to_string(),
            name: file_fmt.name().to_string(),
        };
    }

    /// Handle selection click
    pub(crate) fn handle_selection_click(
        &self,
        offset: usize,
        idx: usize,
        is_shift: bool,
    ) -> Option<(usize, usize)> {
        let current_idx = offset + idx;
        if let Some((select1, select2)) = self.selection.range {
            if is_shift {
                if select1 == current_idx {
                    return Some((current_idx, current_idx));
                } else if current_idx < select1 {
                    return Some((current_idx, select2));
                } else if select1 > current_idx {
                    return Some((current_idx, select1));
                } else if current_idx > select2 || (select1 < current_idx && current_idx < select2)
                {
                    return Some((select1, current_idx));
                }
            } else if select1 == current_idx {
                // unselect
                return None;
            } else {
                // no alt - set a single selection
                return Some((current_idx, current_idx));
            }
        }
        // no previous selection - create new selection
        Some((current_idx, current_idx))
    }

    /// Handle offset click
    pub(crate) fn handle_offset_click(
        &self,
        offset: usize,
        is_shift: bool,
    ) -> Option<(usize, usize)> {
        let end_idx = offset + self.bytes_per_line - 1;
        let end_idx = if self.binary_file.len() > end_idx {
            end_idx
        } else {
            offset + (self.binary_file.len() - offset - 1)
        };
        if let Some((select1, select2)) = self.selection.range {
            if select1 == offset && select2 == end_idx {
                return None;
            }
            if is_shift {
                if offset > select1 {
                    // offset is after
                    return Some((select1, end_idx));
                }
                return Some((offset, select2));
            }
        }
        Some((offset, end_idx))
    }
}

impl DocumentTrait for Document {
    fn path(&self) -> &Path {
        &self.filename
    }
}

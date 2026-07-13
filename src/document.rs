//! Document

use std::ops::RangeInclusive;
use std::path::PathBuf;

use crate::offset::Offset;
use crate::panels::FileInfoData;
use crate::selection::Selection;
use crate::windows::WindowsData;

/// Document
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub(crate) struct Document {
    /// Binary file data
    #[serde(skip)]
    pub(crate) binary_file: Vec<u8>,
    /// Filename of the file
    #[serde(skip)]
    pub(crate) filename: PathBuf,
    /// Selection
    pub(crate) selection: Selection,
    /// Scroll area offset
    pub(crate) offset: Offset,
    /// File info
    #[serde(skip)]
    pub(crate) file_format: Option<FileInfoData>,
    /// Bytes per line
    pub(crate) bytes_per_line: usize,
    /// Windows
    pub(crate) windows_data: WindowsData,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            binary_file: vec![],
            filename: PathBuf::new(),
            selection: Selection::default(),
            offset: Offset::default(),
            file_format: None,
            bytes_per_line: 32,
            windows_data: WindowsData::new(),
        }
    }
}

impl Document {
    /// Go to the selected range
    pub(crate) fn go_to_range(&mut self, range: RangeInclusive<usize>) {
        let start = *range.start();
        self.selection.range = Some((start, *range.end()));
        self.offset.go_to_index(start, self.bytes_per_line);
    }

    /// Mark document as stale
    pub(crate) fn stale(&mut self) {
        self.file_format = None;
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

/// Documents
#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
pub(crate) struct Documents {
    /// current index
    pub(crate) current_idx: usize,
    /// documents
    inner: Vec<Document>,
}

impl Documents {
    /// get current document as mutable
    pub(crate) fn get_current_doc_mut(&mut self) -> Option<&mut Document> {
        if self.inner.is_empty() {
            return None;
        }
        let idx = self.current_idx % self.inner.len();
        Some(&mut self.inner[idx])
    }

    /// add a new document
    pub(crate) fn push(&mut self, document: Document) {
        self.inner.push(document);
        self.current_idx = self.inner.len() - 1;
    }

    /// iter mut on documents
    pub(crate) fn iter_mut(&mut self) -> std::slice::IterMut<'_, Document> {
        self.inner.iter_mut()
    }

    /// Remove a document
    pub(crate) fn remove(&mut self, index: usize) {
        self.inner.remove(index);
        self.current_idx = self.current_idx.saturating_sub(1);
    }

    /// Check if is some
    pub(crate) fn is_some(&self) -> bool {
        !self.inner.is_empty()
    }
}

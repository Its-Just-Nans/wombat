//! Offset

/// Offset of the application
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub(crate) struct Offset {
    /// The current offset
    pub(crate) current: f32,
    /// Does the offset need to be changed
    pub(crate) need_change: bool,
    /// Line to go
    pub(crate) line_to_go: usize,
}

impl Offset {
    /// Go to index
    pub(crate) fn go_to_index(&mut self, start: usize, bytes_per_line: usize) {
        let line_to_go = start / bytes_per_line;
        self.line_to_go = line_to_go;
        self.need_change = true;
    }
}

//! Pdf

use std::path::PathBuf;

use bladvak::AppError;
use pdf::{
    build::{CatalogBuilder, Importer, PageBuilder, PdfBuilder},
    file::FileOptions,
};

use crate::{WombatApp, document::Document};

impl WombatApp {
    /// extract pdf pages
    pub(crate) fn extract_pdf_page(
        &mut self,
        current_idx: usize,
        page_num: u32,
    ) -> Result<(), AppError> {
        let Some(document) = self.documents.get(current_idx) else {
            return Err("Cannot found documents".into());
        };
        let backend = document.binary_file.as_ref().clone();
        let Ok(old_file) = pdf::file::FileOptions::cached().load(backend) else {
            return Err("Cannot load pdf".into());
        };
        let Ok(old_page) = old_file.get_page(page_num) else {
            return Err(format!("Cannot find page {page_num} in PDF").into());
        };

        let mut builder = PdfBuilder::new(FileOptions::cached());

        let mut importer = Importer::new(old_file.resolver(), &mut builder.storage);
        let mut pages = Vec::new();

        let new_page = PageBuilder::clone_page(&old_page, &mut importer)
            .map_err(|e| AppError::new(e.to_string()))?;
        importer
            .finish()
            .verify(&builder.storage.resolver())
            .map_err(|e| AppError::new(e.to_string()))?;

        pages.push(new_page);
        let catalog = CatalogBuilder::from_pages(pages);

        let builder = if let Some(info) = old_file.trailer.info_dict {
            builder.info(info)
        } else {
            builder
        };

        let data = builder
            .build(catalog)
            .map_err(|e| AppError::new(e.to_string()))?;

        self.documents.push(Document::new(
            data,
            PathBuf::from(format!("extracted_page_{page_num}.pdf")),
        ));

        Ok(())
    }
}

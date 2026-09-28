//! Pdf

use std::path::PathBuf;

use bladvak::{AppError, ErrorManager};
use pdf::{
    build::{CatalogBuilder, Importer, PageBuilder, PdfBuilder},
    file::FileOptions,
};

use crate::{WombatApp, document::Document};

impl WombatApp {
    /// extract pdf all pages
    pub(crate) fn extract_pdf_all_pages(
        &mut self,
        current_idx: usize,
        error_manager: &mut ErrorManager,
    ) -> Result<(), AppError> {
        let Some(document) = self.documents.get(current_idx) else {
            return Err("Cannot found documents".into());
        };
        let backend = document.binary_file.as_ref().as_slice();
        let Ok(old_file) = pdf::file::FileOptions::cached().load(backend) else {
            return Err("Cannot load pdf".into());
        };
        let mut docs_to_add = Vec::new();
        for (page_num, one_page) in old_file.pages().enumerate() {
            let Ok(old_page) = one_page else {
                return Err(format!("Cannot find page {page_num} in PDF").into());
            };
            match extract_pdf_single_page(
                &old_page,
                old_file.resolver(),
                old_file.trailer.info_dict.clone(),
                PathBuf::from(format!("extracted_page_{page_num}.pdf")),
            ) {
                Ok(new_doc) => {
                    docs_to_add.push(new_doc);
                }
                Err(err) => {
                    error_manager.add_error(format!("Page {page_num}: {err}"));
                }
            }
        }
        for one_doc in docs_to_add {
            self.documents.push(one_doc);
        }
        Ok(())
    }
}
/// extract pdf page
pub(crate) fn extract_pdf_page(document: &Document, page_num: u32) -> Result<Document, AppError> {
    let backend = document.binary_file.as_ref().clone();
    let Ok(old_file) = pdf::file::FileOptions::cached().load(backend) else {
        return Err("Cannot load pdf".into());
    };
    let Ok(old_page) = old_file.get_page(page_num) else {
        return Err(format!("Cannot find page {page_num} in PDF").into());
    };
    extract_pdf_single_page(
        &old_page,
        old_file.resolver(),
        old_file.trailer.info_dict.clone(),
        PathBuf::from(format!("extracted_page_{page_num}.pdf")),
    )
}
/// extract pdf single page
pub(crate) fn extract_pdf_single_page(
    old_page: &pdf::object::PageRc,
    resolver: impl pdf::object::Resolve,
    info_dict: Option<pdf::object::InfoDict>,
    out_path: PathBuf,
) -> Result<Document, AppError> {
    let mut builder = PdfBuilder::new(FileOptions::cached());

    let mut importer = Importer::new(resolver, &mut builder.storage);
    let mut pages = Vec::new();

    let new_page = PageBuilder::clone_page(old_page, &mut importer)
        .map_err(|e| AppError::new(e.to_string()))?;
    importer
        .finish()
        .verify(&builder.storage.resolver())
        .map_err(|e| AppError::new(e.to_string()))?;

    pages.push(new_page);
    let catalog = CatalogBuilder::from_pages(pages);

    let builder = if let Some(info) = info_dict {
        builder.info(info)
    } else {
        builder
    };

    let data = builder
        .build(catalog)
        .map_err(|e| AppError::new(e.to_string()))?;

    Ok(Document::new(data, out_path))
}

//! Pdf

use std::{path::PathBuf, sync::Arc};

use bladvak::{AppError, ErrorManager};
use hayro::hayro_syntax::Pdf;
use hayro_write::{
    ChunkSettings, ExtractionQuery, extract,
    pdf_writer::{self, Ref},
};

use crate::document::Document;

/// extract pdf all pages
pub(crate) fn extract_pdf_all_pages(
    backend: Arc<Vec<u8>>,
    error_manager: &mut ErrorManager,
) -> Result<Vec<Document>, AppError> {
    let Ok(pdf) = Pdf::new(backend) else {
        return Err("Cannot load pdf".into());
    };
    let mut docs_to_add = Vec::new();
    for (page_num, _one_page) in pdf.pages().iter().enumerate() {
        match extract_pdf_single_page(
            &pdf,
            page_num,
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
    Ok(docs_to_add)
}
/// extract pdf page
pub(crate) fn extract_pdf_page(
    backend: Arc<Vec<u8>>,
    page_num: usize,
) -> Result<Document, AppError> {
    let Ok(pdf) = Pdf::new(backend) else {
        return Err("Cannot load pdf".into());
    };

    extract_pdf_single_page(
        &pdf,
        page_num,
        PathBuf::from(format!("extracted_page_{page_num}.pdf")),
    )
}
/// extract pdf single page
pub(crate) fn extract_pdf_single_page(
    pdf: &Pdf,
    page_num: usize,
    out_path: PathBuf,
) -> Result<Document, AppError> {
    let mut counter = 0;
    let mut new_ref = || {
        counter += 1;
        Ref::new(counter)
    };
    let query = ExtractionQuery::new_page(page_num);
    let Ok(extraction_result) = extract(
        pdf,
        Box::new(&mut new_ref),
        ChunkSettings { pretty: false },
        &[query],
    ) else {
        return Err(format!("Cannot extract pdf page {page_num}").into());
    };

    let chunk = extraction_result.chunk;
    let Some(page_ref) = extraction_result.root_refs.first() else {
        return Err(format!("Cannot extract pdf page {page_num}: no root refs").into());
    };
    let Ok(page_ref) = page_ref else {
        return Err(format!("Cannot extract pdf page {page_num}: no root refs").into());
    };
    let parent = extraction_result.page_tree_parent_ref;

    let catalog_id = new_ref();
    let mut out_pdf = pdf_writer::Pdf::with_settings(ChunkSettings { pretty: false });
    out_pdf.extend(&chunk);
    out_pdf.catalog(catalog_id).pages(parent);
    out_pdf.pages(parent).kids([*page_ref]).count(1);

    let data = out_pdf.finish();
    Ok(Document::new(data, out_path))
}

//! Export PDF (spécification, section 8).

use typst::diag::SourceResult;
use typst_layout::PagedDocument;
use typst_pdf::PdfOptions;

/// Le PDF du document, avec les options par défaut de `typst-pdf`.
pub fn export(document: &PagedDocument) -> SourceResult<Vec<u8>> {
    typst_pdf::pdf(document, &PdfOptions::default())
}

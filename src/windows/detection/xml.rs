//! XML

use bladvak::eframe::egui::{self, CollapsingHeader};
use roxmltree::{Document, Node};

/// xml cached data
#[derive(Debug)]
pub(crate) struct XmlData {
    /// inner xmld string
    inner: String,
}

impl XmlData {
    /// parse the data
    pub(crate) fn parse(binary_data: &[u8]) -> XmlData {
        let xml_str = String::from_utf8_lossy(binary_data);
        XmlData {
            inner: xml_str.to_string(),
        }
    }
}

/// Show XML tree
pub fn xml_tree_ui(ui: &mut egui::Ui, xml: Option<&XmlData>) {
    let Some(xml) = xml else {
        ui.label("Failed to parse xml");
        return;
    };
    match Document::parse(&xml.inner) {
        Ok(doc) => {
            let root = doc.root_element();
            draw_node(ui, root, 0);
        }
        Err(err) => {
            ui.colored_label(egui::Color32::RED, err.to_string());
        }
    }
}

/// Draw node
fn draw_node(ui: &mut egui::Ui, node: Node<'_, '_>, idx: usize) {
    let mut count = idx;
    match node.node_type() {
        roxmltree::NodeType::Element => {
            let label = format_element_label(node);
            count += 1;
            CollapsingHeader::new(&label)
                .id_salt(format!("{label}-{idx}-{count}"))
                .default_open(false)
                .show(ui, |ui| {
                    // Attributes
                    for attr in node.attributes() {
                        ui.label(format!("@{}=\"{}\"", attr.name(), attr.value()));
                    }

                    // Children
                    for child in node.children() {
                        count += 1;
                        draw_node(ui, child, count);
                    }
                });
        }

        roxmltree::NodeType::Text => {
            let text = node.text().unwrap_or("").trim();
            if !text.is_empty() {
                ui.label(text);
            }
        }

        roxmltree::NodeType::Comment => {
            ui.label(format!("<!-- {} -->", node.text().unwrap_or("")));
        }

        _ => {}
    }
}

/// Format element label
fn format_element_label(node: Node<'_, '_>) -> String {
    use std::fmt::Write;
    let mut label = format!("<{}", node.tag_name().name());

    if let Some(id) = node.attribute("id")
        && write!(label, " id=\"{id}\"").is_err()
    {
        return label;
    }

    label.push('>');
    label
}

//! Cert

use bladvak::eframe::egui;
use std::{fmt::Write, ops::RangeInclusive};
use x509_parser::prelude::X509Certificate;
use x509_parser::{pem::Pem, prelude::FromDer};

/// pem data
#[derive(Debug)]
pub(crate) struct CertData {
    /// pem files
    pems: Vec<Pem>,
}

impl CertData {
    /// parse the data
    pub(crate) fn parse(binary_data: &[u8], is_der: bool) -> Option<Self> {
        let pems: Vec<Pem> = if is_der {
            let mut reader = binary_data;
            let mut pems = Vec::new();
            while !reader.is_empty() {
                let (rem, cert) = X509Certificate::from_der(reader).ok()?;
                pems.push(Pem {
                    label: "CERTIFICATE".to_string(),
                    contents: cert.as_raw().to_vec(),
                });
                reader = rem;
            }
            pems
        } else {
            Pem::iter_from_buffer(binary_data)
                .collect::<Result<Vec<_>, _>>()
                .ok()?
        };

        Some(Self { pems })
    }
}

/// show certificates
pub fn show_certs(ui: &mut egui::Ui, opt_data: Option<&CertData>) -> Option<RangeInclusive<usize>> {
    let Some(data) = opt_data else {
        ui.label("Failed to parse pem certificates");
        return None;
    };

    let pems_len = data.pems.len();
    if pems_len > 1 {
        ui.label(format!("Number of certificates: {pems_len}"));
        ui.separator();
    }

    for (idx, one_cert) in data.pems.iter().enumerate() {
        let Ok(x509) = one_cert.parse_x509() else {
            continue;
        };
        egui::Grid::new(format!("cert_grid_{idx}"))
            .striped(true)
            .show(ui, |ui| {
                ui.label("Name");
                ui.label("Data");
                ui.end_row();

                ui.label("Certificate version");
                ui.label(format!("{}", x509.version()));
                ui.end_row();
                ui.label("Certificate serial");
                ui.label(x509.tbs_certificate.serial.to_string());
                ui.end_row();
                ui.label("Certificate signature algorithm");
                ui.label(format!("{}", x509.signature_algorithm.algorithm));
                ui.end_row();
                ui.label("Issuer");
                ui.label(x509.issuer().to_string());
                ui.end_row();
                let is_valid = if x509.tbs_certificate.validity().is_valid() {
                    "certificate is currently valid"
                } else {
                    "certificate is currently invalid"
                };
                ui.label("Validity (not before)");
                ui.label(format!(
                    "{} ({is_valid})",
                    x509.tbs_certificate.validity.not_before
                ));
                ui.end_row();
                ui.label("Validity (not after)");
                ui.label(format!(
                    "{} ({is_valid})",
                    x509.tbs_certificate.validity.not_after
                ));
                ui.end_row();
                ui.label("Subject");
                ui.label(x509.subject().to_string());
                ui.end_row();
                let key = x509.tbs_certificate.subject_pki.subject_public_key.data;
                let key_hex: Option<String> =
                    key.iter()
                        .try_fold(String::with_capacity(key.len() * 2), |mut s, b| {
                            write!(&mut s, "{b:02X}").ok()?;
                            Some(s)
                        });
                if let Some(key) = key_hex {
                    let key_copy = key.clone();
                    let key_hex_trunc = if key.len() > 20 {
                        let mut s = key;
                        s.truncate(20);
                        s.push_str("...");
                        s
                    } else {
                        key
                    };
                    ui.label("Subject public key");
                    ui.label(key_hex_trunc);
                    if ui.button("Copy").clicked() {
                        ui.ctx().copy_text(key_copy);
                    }
                    ui.end_row();
                }
                ui.label("Subject public key algorithm");
                ui.label(format!(
                    "{}",
                    x509.tbs_certificate.subject_pki.algorithm.algorithm
                ));
                ui.end_row();
            });
        if pems_len > 1 {
            ui.separator();
        }
    }
    None
}

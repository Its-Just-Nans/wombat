//! Pdf signatures

use bladvak::{AppError, ErrorManager};

use bladvak::eframe::egui::{self, Color32};
use cms::der::{DateTime, Decode, Encode, asn1::OctetString};
use cms::{content_info::ContentInfo, signed_data::SignedData};
use rsa::{
    PublicKey,
    pkcs1v15::{Signature, VerifyingKey},
    pkcs8::DecodePublicKey,
};
use sha1::Sha1;
use sha2::{Digest, Sha256};
use signature::Verifier;
use std::error::Error;
use std::ops::RangeInclusive;
use std::path::PathBuf;
use std::time::{Duration, UNIX_EPOCH};
use time::OffsetDateTime;
use x509_cert::certificate::CertificateInner;

use crate::document::Document;

/// message digest oid
const MESSAGE_DIGEST_OID: &str = "1.2.840.113549.1.9.4";
/// sha256
const SHA256_OID: &str = "2.16.840.1.101.3.4.2.1";
/// sha1 oid
const SHA1_OID: &str = "1.3.14.3.2.26";

/// Byte Range result
#[derive(Debug)]
pub(crate) struct ByteRangeResult {
    /// range of the signature
    pub(crate) range: [usize; 4],
    /// results
    pub(crate) results: Vec<SignResult>,
}

/// Signature result
#[derive(Debug)]
pub(crate) struct SignResult {
    /// certificate
    pub(crate) certificate: CertificateInner,
    /// is valid
    pub(crate) is_valid: bool,
}

/// Verify PDF signature
#[allow(clippy::too_many_lines)]
fn verify_pdf_signature(
    pdf: &[u8],
    byte_range: [usize; 4],
) -> Result<Vec<SignResult>, Box<dyn Error>> {
    // 1. Reconstruct the signed PDF bytes from /ByteRange.
    let [offset1, length1, offset2, length2] = byte_range;

    let end1 = offset1.checked_add(length1).ok_or("ByteRange overflow")?;
    let end2 = offset2.checked_add(length2).ok_or("ByteRange overflow")?;

    if end1 > pdf.len() || end2 > pdf.len() || offset2 < end1 {
        return Err("Invalid PDF ByteRange".into());
    }

    let mut signed_pdf = Vec::with_capacity(length1 + length2);
    signed_pdf.extend_from_slice(&pdf[offset1..end1]);
    signed_pdf.extend_from_slice(&pdf[offset2..end2]);

    // The excluded region normally contains <hex-encoded CMS data>.
    // This assumes offset2 begins immediately after the closing '>'.
    let contents_start = end1.checked_add(1).ok_or("Offset overflow")?;
    let contents_end = offset2.checked_sub(1).ok_or("Offset underflow")?;

    if contents_start > contents_end || contents_end > pdf.len() {
        return Err("Invalid PDF signature Contents range".into());
    }

    let hex_contents = &pdf[contents_start..contents_end];

    // 2. Decode the PDF hexadecimal Contents field.
    let hex_contents = std::str::from_utf8(hex_contents)?;
    let cms_bytes = hex::decode(hex_contents.trim())?;

    // 3. Remove padding by parsing only the first DER object.
    let der_len = der_object_length(&cms_bytes)?;
    let content_info = ContentInfo::from_der(&cms_bytes[..der_len])?;

    // 4. Decode SignedData.
    let signed_data: SignedData = content_info.content.decode_as::<SignedData>()?;

    let signer = signed_data
        .signer_infos
        .0
        .get(0)
        .ok_or("CMS contains no signers")?;

    if ![SHA256_OID, SHA1_OID].contains(&signer.digest_alg.oid.to_string().as_str()) {
        return Err(format!(
            "{} is not supported by this implementation {SHA1_OID}",
            signer.digest_alg.oid
        )
        .into());
    }

    let attrs = signer
        .signed_attrs
        .as_ref()
        .ok_or("CMS signer has no authenticated attributes")?;

    // 5. Locate messageDigest.
    let mut expected_digest: Option<Vec<u8>> = None;

    for attr in attrs.iter() {
        if attr.oid.to_string() == MESSAGE_DIGEST_OID {
            let value = attr
                .values
                .iter()
                .next()
                .ok_or("messageDigest attribute has no value")?;

            expected_digest = Some(value.decode_as::<OctetString>()?.as_bytes().to_vec());
            break;
        }
    }

    let expected_digest = expected_digest.ok_or("messageDigest attribute missing")?;

    // 6. Verify PDF document integrity.
    match signer.digest_alg.oid.to_string().as_ref() {
        SHA1_OID => {
            if Sha1::digest(&signed_pdf).as_slice() != expected_digest.as_slice() {
                return Err("Invalid digest".into());
            }
        }
        SHA256_OID => {
            if Sha256::digest(&signed_pdf).as_slice() != expected_digest.as_slice() {
                return Err("Invalid digest".into());
            }
        }
        _ => {
            return Err("Not supported".into());
        }
    }

    // 7. Find an RSA certificate in the CMS certificate set.
    let cert_set = signed_data
        .certificates
        .as_ref()
        .ok_or("CMS contains no certificates")?;
    let mut result = Vec::new();
    for choice in cert_set.0.iter() {
        if let cms::cert::CertificateChoices::Certificate(certificate) = choice {
            let tbs_certificate = certificate.tbs_certificate();
            let spki_der = tbs_certificate.subject_public_key_info().to_der()?;

            let is_valid = if let Ok(public_key) = PublicKey::from_public_key_der(&spki_der) {
                // 8. Verify the authenticated attributes signature.
                // CMS signatures cover the DER encoding of SignedAttributes as SET OF.
                let attrs_der = attrs.to_der()?;
                let signature = Signature::try_from(signer.signature.as_bytes())?;

                let validation_result = match signer.digest_alg.oid.to_string().as_ref() {
                    SHA1_OID => {
                        let verifier = VerifyingKey::<Sha1>::new(public_key);

                        verifier.verify(&attrs_der, &signature)
                    }
                    SHA256_OID => {
                        let verifier = VerifyingKey::<Sha256>::new(public_key);

                        verifier.verify(&attrs_der, &signature)
                    }
                    _ => {
                        return Err("Not supported".into());
                    }
                };
                validation_result.is_ok()
            } else {
                false
            };
            result.push(SignResult {
                certificate: certificate.clone(),
                is_valid,
            });
        }
    }
    Ok(result)
}

/// check the der object length
fn der_object_length(data: &[u8]) -> Result<usize, &'static str> {
    if data.len() < 2 {
        return Err("DER data too short");
    }

    let first_len = data[1] as usize;

    let (header_len, content_len) = if first_len < 128 {
        (2, first_len)
    } else {
        let num_len_bytes = first_len & 0x7f;

        if num_len_bytes == 0 || num_len_bytes > 4 {
            return Err("Invalid DER length");
        }

        if data.len() < 2 + num_len_bytes {
            return Err("Incomplete DER length");
        }

        let mut len = 0usize;
        for &b in &data[2..2 + num_len_bytes] {
            len = (len << 8) | b as usize;
        }

        (2 + num_len_bytes, len)
    };

    let total_len = header_len
        .checked_add(content_len)
        .ok_or("DER length overflow")?;

    if total_len > data.len() {
        return Err("Incomplete DER object");
    }

    Ok(total_len)
}

/// parse a byte range
fn parse_byte_range(input: &[u8]) -> Result<[u64; 4], String> {
    let start = input.iter().position(|&b| b == b'[').ok_or("Missing '['")?;

    let end = input[start..]
        .iter()
        .position(|&b| b == b']')
        .map(|i| start + i)
        .ok_or("Missing ']'")?;

    let values: Vec<u64> = input[start + 1..end]
        .split(|b| b.is_ascii_whitespace() || *b == b'.')
        .filter(|s| !s.is_empty())
        .map(|s| {
            std::str::from_utf8(s)
                .map_err(|e| e.to_string())?
                .parse::<u64>()
                .map_err(|e| e.to_string())
        })
        .collect::<Result<_, String>>()?;

    values
        .try_into()
        .map_err(|v: Vec<u64>| format!("Expected 4 values, got {}", v.len()))
}

/// find byte ranges
fn find_byte_ranges(input: &[u8]) -> Vec<[u64; 4]> {
    let marker = b"/ByteRange";
    let mut ranges = Vec::new();
    let mut pos = 0;

    while pos + marker.len() <= input.len() {
        let Some(offset) = input[pos..].windows(marker.len()).position(|w| w == marker) else {
            break;
        };

        pos += offset + marker.len();

        // Find the opening bracket.
        let Some(open_offset) = input[pos..].iter().position(|&b| b == b'[') else {
            break;
        };

        let open = pos + open_offset;

        // Find the closing bracket.
        let Some(close_offset) = input[open..].iter().position(|&b| b == b']') else {
            break;
        };

        let close = open + close_offset;

        if let Ok(range) = parse_byte_range(&input[open..=close]) {
            ranges.push(range);
        }

        pos = close + 1;
    }

    ranges
}

#[derive(Debug)]
pub(crate) struct PdfSignaturesData {
    /// bytes ranges
    inner: Vec<ByteRangeResult>,
}

impl PdfSignaturesData {
    /// extract pdf images
    /// Not all algorithms supported
    pub(crate) fn try_parse(input: &[u8]) -> Result<Self, AppError> {
        let mut range_results = Vec::new();
        for range in find_byte_ranges(input) {
            let range = [
                usize::try_from(range[0]).map_err(|e| e.to_string())?,
                usize::try_from(range[1]).map_err(|e| e.to_string())?,
                usize::try_from(range[2]).map_err(|e| e.to_string())?,
                usize::try_from(range[3]).map_err(|e| e.to_string())?,
            ];
            let results = verify_pdf_signature(input, range).map_err(|e| e.to_string())?;
            range_results.push(ByteRangeResult { range, results });
        }
        Ok(Self {
            inner: range_results,
        })
    }

    /// show ui
    pub(crate) fn ui(
        &self,
        ui: &mut egui::Ui,
        error_manager: &mut ErrorManager,
    ) -> (Option<RangeInclusive<usize>>, Option<Document>) {
        let mut go_to_range = None;
        let mut new_doc = None;
        if self.inner.is_empty() {
            ui.label("No ByteRange found");
        }
        for one_byte_range_res in &self.inner {
            let head_text = format!("Bytes range {:?}", one_byte_range_res.range);
            ui.collapsing(head_text, |ui| {
                if ui.button("Show").clicked() {
                    let start = one_byte_range_res.range[0]
                        .saturating_add(one_byte_range_res.range[1].saturating_add(1));
                    let end = one_byte_range_res.range[2].saturating_sub(2);
                    go_to_range = Some(start..=end);
                }
                if one_byte_range_res.results.is_empty() {
                    ui.label("No signature results");
                }
                for (idx, one_res) in one_byte_range_res.results.iter().enumerate() {
                    ui.collapsing(format!("Certificate {idx}"), |ui| {
                        ui.label(format!(
                            "Subject: {}",
                            one_res.certificate.tbs_certificate().subject()
                        ));
                        ui.label(format!(
                            "Issuer: {}",
                            one_res.certificate.tbs_certificate().issuer()
                        ));
                        let validity = one_res.certificate.tbs_certificate().validity();
                        let now = OffsetDateTime::now_utc();
                        #[allow(clippy::cast_precision_loss)]
                        #[allow(clippy::cast_possible_truncation)]
                        #[allow(clippy::cast_sign_loss)]
                        let system_time = if now.unix_timestamp_nanos() >= 0 {
                            UNIX_EPOCH + Duration::from_nanos(now.unix_timestamp_nanos() as u64)
                        } else {
                            UNIX_EPOCH - Duration::from_nanos((-now.unix_timestamp_nanos()) as u64)
                        };
                        let Ok(current_datetime) = DateTime::from_system_time(system_time) else {
                            ui.label("Error cannot get current time");
                            return;
                        };
                        ui.horizontal(|ui| {
                            ui.label(format!("Not before: {} -", validity.not_before));
                            let is_valid = validity.not_before.to_date_time() < current_datetime;
                            ui.colored_label(
                                if is_valid {
                                    Color32::GREEN
                                } else {
                                    Color32::RED
                                },
                                if is_valid { "true" } else { "false" },
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label(format!("Not after: {} -", validity.not_after));
                            let is_valid = validity.not_after.to_date_time() > current_datetime;
                            ui.colored_label(
                                if is_valid {
                                    Color32::GREEN
                                } else {
                                    Color32::RED
                                },
                                if is_valid { "true" } else { "false" },
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("Is signature valid:");
                            ui.colored_label(
                                if one_res.is_valid {
                                    Color32::GREEN
                                } else {
                                    Color32::RED
                                },
                                if one_res.is_valid { "true" } else { "false" },
                            );
                        });
                        if ui.button("Open in a new document").clicked() {
                            match one_res.certificate.to_der() {
                                Ok(der) => {
                                    let filename = PathBuf::from("certificate.pem");
                                    let mut doc = Document::new(der, filename);
                                    doc.windows_data.parsing.is_open = true;
                                    new_doc = Some(doc);
                                }
                                Err(err) => {
                                    error_manager.add_error(err.to_string());
                                }
                            }
                        }
                    });
                }
            });
        }
        (go_to_range, new_doc)
    }
}

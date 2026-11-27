//! PGP

use std::ops::RangeInclusive;

use bladvak::{
    ErrorManager,
    eframe::egui::{self},
};

use crate::document::Document;

/// PGP Raw data
#[derive(Debug)]
pub(crate) struct PgpRawData {
    /// packets
    inner: Vec<Packet>,
}

impl PgpRawData {
    /// Parse PGP packets
    pub(crate) fn try_parse(binary_file: &[u8]) -> Result<Self, String> {
        let mut stream = Stream::new(binary_file, 0, binary_file.len());

        let mut packets = Vec::new();

        while stream.pos < stream.end {
            packets.push(Packet::new(&mut stream)?);
        }
        Ok(Self { inner: packets })
    }

    /// show ui
    pub(crate) fn ui(
        &self,
        ui: &mut egui::Ui,
        _binary_file: &[u8],
        _error_manager: &mut ErrorManager,
    ) -> (Option<RangeInclusive<usize>>, Option<Document>) {
        let mut new_range = None;
        for p in &self.inner {
            let Packet {
                start,
                end,
                tag,
                length,
                length_size,
                inner,
            } = p;
            ui.horizontal(|ui| {
                ui.label(inner.name());
                if ui.button("Show").clicked() {
                    new_range = Some(*start..=*end);
                }
            });
            ui.horizontal(|ui| {
                ui.label(format!("Tag: {tag}"));
                if ui.button("Show").clicked() {
                    new_range = Some(*start..=*start);
                }
            });
            ui.horizontal(|ui| {
                ui.label(format!("Length: {length}"));
                if ui.button("Show").clicked() {
                    let rstart = start.saturating_add(1);
                    let end = start.saturating_add(*length_size);
                    new_range = Some(rstart..=end);
                }
            });
            if let Some(r) = inner.ui(ui) {
                new_range = Some(r);
            };
            ui.label(format!("{p:?}"));
            ui.separator();
        }
        (new_range, None)
    }
}

#[derive(Debug)]
pub struct Stream<'a> {
    pub bytes: &'a [u8],
    pub start: usize,
    pub end: usize,
    pub length_headers: Vec<usize>,
    pub pos: usize,
}

impl<'a> Stream<'a> {
    pub fn new(bytes: &'a [u8], start: usize, end: usize) -> Self {
        Self {
            bytes,
            start,
            end,
            length_headers: Vec::new(),
            pos: start,
        }
    }

    pub fn remaining(&self) -> usize {
        self.end.saturating_sub(self.pos)
    }

    pub fn to_array(&self) -> &[u8] {
        &self.bytes[self.start..self.end]
    }

    pub fn octet(&mut self) -> Option<u8> {
        if self.pos >= self.end {
            return None;
        }

        while self.length_headers.first() == Some(&self.pos) {
            self.pos += 1;
            self.length_headers.remove(0);
        }

        if self.pos >= self.end {
            return None;
        }

        let value = self.bytes[self.pos];
        self.pos += 1;

        Some(value)
    }

    pub fn uint(&mut self, n: usize) -> Option<u32> {
        let mut ret = 0u32;

        for _ in 0..n {
            ret = (ret << 8) | self.octet()? as u32;
        }

        Some(ret)
    }

    pub fn uint16(&mut self) -> Option<u16> {
        self.uint(2).map(|v| v as u16)
    }

    pub fn uint32(&mut self) -> Option<u32> {
        self.uint(4)
    }

    pub fn time(&mut self) -> Option<Timestamp> {
        let timestamp = self.uint32()?;

        Some(if timestamp > 0 {
            Timestamp::Unix(timestamp)
        } else {
            Timestamp::Never
        })
    }

    pub fn hex(&mut self, n: usize) -> Option<String> {
        const HEX: &[u8; 16] = b"0123456789abcdef";

        let mut output = String::with_capacity(n * 2);

        for _ in 0..n {
            let byte = self.octet()?;

            output.push(HEX[(byte >> 4) as usize] as char);
            output.push(HEX[(byte & 0x0f) as usize] as char);
        }

        Some(output)
    }

    pub fn utf8(&mut self, n: usize) -> Option<String> {
        let mut bytes = Vec::with_capacity(n);

        for _ in 0..n {
            bytes.push(self.octet()?);
        }

        String::from_utf8(bytes).ok()
    }

    pub fn multi_precision_integer(&mut self) -> Option<String> {
        let bit_length = self.uint16()? as usize;
        let byte_length = (bit_length + 7) / 8;

        self.hex(byte_length)
    }

    // RFC 4880 3.7.1.3
    pub fn iteration_count(&mut self) -> Option<u32> {
        let c = self.octet()?;

        Some((16 + (c as u32 & 15)) << ((c >> 4) + 6))
    }

    // RFC 4880 4.2.2, 5.2.3.1
    pub fn variable_length_length(
        &mut self,
        support_partial: bool,
        mark: bool,
    ) -> Option<(usize, usize)> {
        let mut count = 0;
        let mut length = self.octet()? as usize;
        count += 1;

        if mark {
            self.length_headers.push(self.pos - 1);
        }

        if length < 192 {
            return Some((length, count));
        }

        if length < if support_partial { 224 } else { 255 } {
            length = ((length - 192) << 8) + self.octet()? as usize + 192;
            count += 1;

            if mark {
                self.length_headers.push(self.pos - 1);
            }

            return Some((length, count));
        }

        // Partial lengths
        if length < 255 {
            let next = 1usize << (length & 0x1f);
            let current_pos = self.pos;

            self.pos = self.pos.checked_add(next + 1)?;

            let (next_jump, new_count) = self.variable_length_length(support_partial, true)?;
            count += new_count;

            self.pos = current_pos;

            return Some((next + next_jump, count));
        }

        length = self.uint32()? as usize;
        count += 1;

        if mark {
            self.length_headers.push(self.pos - 1);
            self.length_headers.push(self.pos - 2);
            self.length_headers.push(self.pos - 3);
            self.length_headers.push(self.pos - 4);
        }

        Some((length, count))
    }

    pub fn sub_parse<F>(&mut self, n: usize, f: F) -> bool
    where
        F: FnOnce(&mut Self),
    {
        let old_end = self.end;
        let old_start = self.start;

        self.start = self.pos;
        self.end = self.pos.saturating_add(n).min(old_end);

        f(self);

        let incomplete = self.pos != self.end;

        if incomplete {
            self.pos = self.end;
        }

        self.start = old_start;
        self.end = old_end;

        incomplete
    }

    pub fn lookup(&mut self, table: &[Option<&'static str>]) -> Option<LookupResult> {
        let octet = self.octet()?;

        Some(LookupResult {
            string: table.get(octet as usize).copied().flatten(),
            id: octet,
            critical: false,
            raw: octet,
        })
    }

    pub fn lookup_critical(&mut self, table: &[Option<&'static str>]) -> Option<LookupResult> {
        let raw = self.octet()?;
        let critical = (raw & 0x80) != 0;
        let normalized = raw & !0x80;

        Some(LookupResult {
            string: table.get(normalized as usize).copied().flatten(),
            id: normalized,
            critical,
            raw,
        })
    }

    pub fn lookup_array(
        &mut self,
        table: &[Option<&'static str>],
        n: usize,
    ) -> Option<Vec<LookupResult>> {
        let mut results = Vec::with_capacity(n);

        for _ in 0..n {
            results.push(self.lookup(table)?);
        }

        Some(results)
    }

    pub fn lookup_flags(
        &mut self,
        table: &[Option<&'static str>],
        n: usize,
    ) -> Option<Vec<LookupResult>> {
        let flags = self.uint(n)?;
        let mut results = Vec::new();

        for bit in 0..(n * 8) {
            let mask = 1u32 << bit;

            if flags & mask != 0 {
                results.push(LookupResult {
                    string: table.get(mask as usize).copied().flatten(),
                    id: mask as u8,
                    critical: false,
                    raw: mask as u8,
                });
            }
        }

        Some(results)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Timestamp {
    Unix(u32),
    Never,
}

#[derive(Debug, Clone)]
pub struct LookupResult {
    pub string: Option<&'static str>,
    pub id: u8,
    pub critical: bool,
    pub raw: u8,
}

impl LookupResult {
    pub fn to_string(&self) -> String {
        let name = self.string.unwrap_or("undefined");

        if self.critical {
            format!("{} (0x{:x})* (critical: 0x{:x})", name, self.id, self.raw)
        } else {
            format!("{} (0x{:x})", name, self.id)
        }
    }
}

/// Packet
#[derive(Debug)]
pub struct Packet {
    /// start
    start: usize,
    /// end
    end: usize,
    /// tag
    tag: u8,
    /// length
    length: usize,
    /// number of bytes for the length
    length_size: usize,
    /// actual packet
    inner: InnerPacket,
}

impl Packet {
    /// New packet
    pub fn new(stream: &mut Stream<'_>) -> Result<Self, String> {
        let start = stream.pos;
        let octet = stream.octet().ok_or("no")?;
        if (octet & 0x80) != 0x80 {
            return Err(format!("Invalid packet format: {octet:x}"));
        }
        let (tag, length, length_size) = if octet & 0x40 == 0x40 {
            // new header
            let (length, length_size) = stream
                .variable_length_length(true, false)
                .ok_or("Cannot get len")?;
            (octet & 0x3f, length, length_size)
        } else {
            // old header
            let size = octet & 0x3;
            let tag = (octet & 0x3c) >> 2;
            let (length, length_size) = match size {
                2 => (stream.uint32().ok_or("Cannot get len")? as usize, 4),
                1 => (stream.uint16().ok_or("Cannot get len")? as usize, 2),
                0 => (stream.octet().ok_or("Cannot get len")? as usize, 1),
                _ => (0, 0),
            };
            (tag, length, length_size)
        };
        let inner = InnerPacket::parse_from_tag(stream, tag, length)?;
        let end = start + length_size + length; //stream.pos;
        stream.pos = end + 1;
        Ok(Self {
            start,
            end,
            tag,
            length,
            length_size,
            inner,
        })
    }
}

macro_rules! lookup_fn {
    ($name:ident, $ty:ty, {$($id:expr => $text:expr,)*}) => {
        pub fn $name(id: $ty) -> &'static str {
            match id {
                $($id => $text,)*
                _ => "Unknown",
            }
        }
    };
}

lookup_fn!(public_key_algorithm_to_text, u8, {
    0 => "Reserved",
    1 => "RSA (Encrypt or Sign)",
    2 => "RSA Encrypt-Only",
    3 => "RSA Sign-Only",
    16 => "Elgamal (Encrypt-Only)",
    17 => "DSA (Digital Signature Algorithm)",
    18 => "ECDH public key algorithm",
    19 => "ECDSA public key algorithm",
    20 => "Reserved (formerly Elgamal Encrypt or Sign)",
    21 => "Reserved for Diffie-Hellman (X9.42, as defined for IETF/S-MIME)",
    22 => "EdDSALegacy (deprecated)",
    23 => "Reserved (AEDH)",
    24 => "Reserved (AEDSA)",
    25 => "X25519",
    26 => "X448",
    27 => "Ed25519",
    28 => "Ed448",
    100 => "Private or Experimental Use",
    101 => "Private or Experimental Use",
    102 => "Private or Experimental Use",
    103 => "Private or Experimental Use",
    104 => "Private or Experimental Use",
    105 => "Private or Experimental Use",
    106 => "Private or Experimental Use",
    107 => "Private or Experimental Use",
    108 => "Private or Experimental Use",
    109 => "Private or Experimental Use",
    110 => "Private or Experimental Use",
});

lookup_fn!(symmetric_key_algorithm_to_text, u8, {
    0 => "Plaintext or unencrypted data",
    1 => "IDEA",
    2 => "TripleDES (or DES-EDE) with 168-bit key derived from 192",
    3 => "CAST5 with 128-bit key",
    4 => "Blowfish with 128-bit key, 16 rounds",
    5 => "Reserved",
    6 => "Reserved",
    7 => "AES with 128-bit key",
    8 => "AES with 192-bit key",
    9 => "AES with 256-bit key",
    10 => "Twofish with 256-bit key",
    11 => "Camellia with 128-bit key",
    12 => "Camellia with 192-bit key",
    13 => "Camellia with 256-bit key",
    100 => "Private or Experimental Use",
    101 => "Private or Experimental Use",
    102 => "Private or Experimental Use",
    103 => "Private or Experimental Use",
    104 => "Private or Experimental Use",
    105 => "Private or Experimental Use",
    106 => "Private or Experimental Use",
    107 => "Private or Experimental Use",
    108 => "Private or Experimental Use",
    109 => "Private or Experimental Use",
    110 => "Private or Experimental Use",
    253 => "Reserved to avoid collision with Secret Key Encryption",
    254 => "Reserved to avoid collision with Secret Key Encryption",
    255 => "Reserved to avoid collision with Secret Key Encryption",
});

lookup_fn!(secret_key_algorithm_block_size, u8, {
    1 => "8",
    2 => "8",
    3 => "8",
    4 => "8",
    7 => "16",
    8 => "16",
    9 => "16",
    10 => "16",
    11 => "16",
    12 => "16",
    13 => "16",
});

lookup_fn!(aead_algorithm_to_text, u8, {
    0 => "Reserved",
    1 => "EAX",
    2 => "OCB",
    3 => "GCM",
    100 => "Private or Experimental Use",
    101 => "Private or Experimental Use",
    102 => "Private or Experimental Use",
    103 => "Private or Experimental Use",
    104 => "Private or Experimental Use",
    105 => "Private or Experimental Use",
    106 => "Private or Experimental Use",
    107 => "Private or Experimental Use",
    108 => "Private or Experimental Use",
    109 => "Private or Experimental Use",
    110 => "Private or Experimental Use",
});

lookup_fn!(aead_iv_length, u8, {
    1 => "16",
    2 => "15",
    3 => "12",
});

lookup_fn!(aead_authentication_tag_length, u8, {
    1 => "16",
    2 => "16",
    3 => "16",
});

lookup_fn!(compression_algorithm_to_text, u8, {
    0 => "Uncompressed",
    1 => "ZIP",
    2 => "ZLIB",
    3 => "BZip2",
    100 => "Private or Experimental Use",
    101 => "Private or Experimental Use",
    102 => "Private or Experimental Use",
    103 => "Private or Experimental Use",
    104 => "Private or Experimental Use",
    105 => "Private or Experimental Use",
    106 => "Private or Experimental Use",
    107 => "Private or Experimental Use",
    108 => "Private or Experimental Use",
    109 => "Private or Experimental Use",
    110 => "Private or Experimental Use",
});

lookup_fn!(hash_salt_size, u8, {
    8 => "16",
    9 => "24",
    10 => "32",
    11 => "16",
    12 => "16",
    14 => "32",
    100 => "-1",
    101 => "-1",
    102 => "-1",
    103 => "-1",
    104 => "-1",
    105 => "-1",
    106 => "-1",
    107 => "-1",
    108 => "-1",
    109 => "-1",
    110 => "-1",
});

lookup_fn!(signature_type_to_text, u8, {
    0 => "Signature of a binary document.",
    1 => "Signature of a canonical text document.",
    2 => "Standalone signature.",
    16 => "Generic certification of a User ID and Public-Key packet.",
    17 => "Persona certification of a User ID and Public-Key packet.",
    18 => "Casual certification of a User ID and Public-Key packet.",
    19 => "Positive certification of a User ID and Public-Key packet.",
    24 => "Subkey Binding Signature",
    25 => "Primary Key Binding Signature",
    31 => "Direct Key Signature",
    32 => "Key revocation signature",
    40 => "Subkey revocation signature",
    48 => "Certification revocation signature",
    64 => "Timestamp signature.",
    80 => "Third-Party Confirmation signature.",
    255 => "Reserved",
});

lookup_fn!(signature_subpacket_type_to_text, u8, {
    0 => "Reserved",
    1 => "Reserved",
    2 => "Signature Creation Time",
    3 => "Signature Expiration Time",
    4 => "Exportable Certification",
    5 => "Trust Signature",
    6 => "Regular Expression",
    7 => "Revocable",
    8 => "Reserved",
    9 => "Key Expiration Time",
    10 => "Placeholder for backward compatibility",
    11 => "Preferred Symmetric Ciphers for v1 SEIPD",
    12 => "Revocation Key (deprecated)",
    13 => "Reserved",
    14 => "Reserved",
    15 => "Reserved",
    16 => "Issuer",
    17 => "Reserved",
    18 => "Reserved",
    19 => "Reserved",
    20 => "Notation Data",
    21 => "Preferred Hash Algorithms",
    22 => "Preferred Compression Algorithms",
    23 => "Key Server Preferences",
    24 => "Preferred Key Server",
    25 => "Primary User ID",
    26 => "Policy URI",
    27 => "Key Flags",
    28 => "Signer's User ID",
    29 => "Reason for Revocation",
    30 => "Features",
    31 => "Signature Target",
    32 => "Embedded Signature",
    33 => "Issuer Fingerprint",
    34 => "Reserved (Formerly Preferred AEAD Algorithms)",
    35 => "Intended Recipient Fingerprint",
    37 => "Reserved (Attested Certifications)",
    38 => "Reserved (Key Block)",
    39 => "Preferred AEAD Ciphersuites",
    100 => "Private or Experimental Use",
    101 => "Private or Experimental Use",
    102 => "Private or Experimental Use",
    103 => "Private or Experimental Use",
    104 => "Private or Experimental Use",
    105 => "Private or Experimental Use",
    106 => "Private or Experimental Use",
    107 => "Private or Experimental Use",
    108 => "Private or Experimental Use",
    109 => "Private or Experimental Use",
    110 => "Private or Experimental Use",
});

lookup_fn!(keyserver_preference_to_text, u8, {
    128 => "No-modify",
});

lookup_fn!(key_flag_to_text, u16, {
    0x0001 => "certify",
    0x0002 => "sign",
    0x0004 => "encrypt communications",
    0x0008 => "encrypt storage",
    0x0010 => "split key",
    0x0020 => "authentication",
    0x0080 => "shared key",
    0x4000 => "reserved (ASDK)",
    0x8000 => "reserved (timestamping)",
});

lookup_fn!(key_feature_to_text, u8, {
    1 => "Modification detection v1",
    2 => "Reserved",
    4 => "Reserved",
    8 => "Modification detection v2",
});

lookup_fn!(string_to_key_usage_to_text, u8, {
    0 => "Plaintext or unencrypted data",
    1 => "IDEA",
    2 => "TripleDES (or DES-EDE) with 168-bit key derived from 192",
    3 => "CAST5 with 128-bit key",
    4 => "Blowfish with 128-bit key, 16 rounds",
    5 => "Reserved",
    6 => "Reserved",
    7 => "AES with 128-bit key",
    8 => "AES with 192-bit key",
    9 => "AES with 256-bit key",
    10 => "Twofish with 256-bit key",
    11 => "Camellia with 128-bit key",
    12 => "Camellia with 192-bit key",
    13 => "Camellia with 256-bit key",
    100 => "Private or Experimental Use",
    101 => "Private or Experimental Use",
    102 => "Private or Experimental Use",
    103 => "Private or Experimental Use",
    104 => "Private or Experimental Use",
    105 => "Private or Experimental Use",
    106 => "Private or Experimental Use",
    107 => "Private or Experimental Use",
    108 => "Private or Experimental Use",
    109 => "Private or Experimental Use",
    110 => "Private or Experimental Use",
    253 => "AEAD",
    254 => "CFB",
    255 => "MalleableCFB",
});

lookup_fn!(string_to_key_specifier_to_text, u8, {
    0 => "Simple S2K",
    1 => "Salted S2K",
    2 => "Reserved value",
    3 => "Iterated and Salted S2K",
    4 => "Argon2",
    100 => "Private or Experimental Use",
    101 => "Private or Experimental Use",
    102 => "Private or Experimental Use",
    103 => "Private or Experimental Use",
    104 => "Private or Experimental Use",
    105 => "Private or Experimental Use",
    106 => "Private or Experimental Use",
    107 => "Private or Experimental Use",
    108 => "Private or Experimental Use",
    109 => "Private or Experimental Use",
    110 => "Private or Experimental Use",
});

lookup_fn!(literal_data_format_to_text, u8, {
    0x01 => "local",
    0x31 => "local",
    0x62 => "binary",
    0x74 => "text",
    0x75 => "utf8",
});

lookup_fn!(notation_data_flag_to_text, u32, {
    0x8000_0000 => "human-readable",
});

lookup_fn!(revocation_reason_to_text, u8, {
    0 => "No reason specified",
    1 => "Key is superseded",
    2 => "Key material has been compromised",
    3 => "Key is retired and no longer used",
    32 => "User ID information is no longer valid",
    100 => "Private Use",
    101 => "Private Use",
    102 => "Private Use",
    103 => "Private Use",
    104 => "Private Use",
    105 => "Private Use",
    106 => "Private Use",
    107 => "Private Use",
    108 => "Private Use",
    109 => "Private Use",
    110 => "Private Use",
});

#[derive(Debug)]
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct RsaKey {
    n: StartPosition,
    e: StartPosition,
}

#[derive(Debug)]
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) enum PublicKey {
    Reserved,
    RsaEncryptOrSign(RsaKey),
    RsaEncryptOnly(RsaKey),
    RsaSignOnly(RsaKey),
    ElGamal((usize, usize, usize)),
    Dsa,
    Ecdh,
    Ecdsa,
    ReservedElGamal,
    ReservedDiffieHellman,
    EdDsaLegacy,
    ReservedAeDH,
    ReservedAeDSA,
    X25519,
    X448,
    Ed25519,
    Ed448,
    Unknown(u8),
}

impl PublicKey {
    /// parse
    fn parse_from_tag(buffer: &mut Stream<'_>, tag: u8) -> Self {
        match tag {
            0 => Self::Reserved,
            1 => Self::RsaEncryptOrSign(RsaKey { n: 0, e: 0 }),
            2 => Self::RsaEncryptOnly(RsaKey { n: 0, e: 0 }),
            3 => Self::RsaSignOnly(RsaKey { n: 0, e: 0 }),
            16 => Self::ElGamal((0, 0, 0)),
            17 => Self::Dsa,
            18 => Self::Ecdh,
            19 => Self::Ecdsa,
            20 => Self::ReservedElGamal,
            21 => Self::ReservedDiffieHellman,
            22 => Self::EdDsaLegacy,
            23 => Self::ReservedAeDH,
            24 => Self::ReservedAeDSA,
            25 => Self::X25519,
            26 => Self::X448,
            27 => Self::Ed25519,
            28 => Self::Ed448,
            _ => Self::Unknown(tag),
        }
    }

    /// Show ui
    fn ui(&self, ui: &mut egui::Ui) {
        match self {
            Self::RsaEncryptOrSign(d) | Self::RsaEncryptOnly(d) | Self::RsaSignOnly(d) => {
                ui.label(format!("{:?}", d));
            }
            Self::ElGamal(d) => {
                ui.label(format!("{:?}", d));
            }
            _ => {
                ui.label("TODO");
            }
        }
    }
}

type StartPosition = usize;

#[derive(Debug)]
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct PublicKeyPacket {
    version: u8,
    created_at: Timestamp,
    valid_days: Option<u16>,
    algorithm: u8,
    octet_count: Option<u32>,
    key: PublicKey,
}

#[derive(Debug)]
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) enum InnerPacket {
    ReservedReally,
    PublicKeyEncryptedSessionKeyPacket,
    SignaturePacket,
    SymmetricKeyEncryptedSessionKeyPacket,
    OnePassSignaturePacket,
    SecretKeyPacket,
    PublicKeyPacket((StartPosition, PublicKeyPacket)),
    SecretSubKeyPacket,
    CompressedDataPacket,
    SymmetricallyEncryptedDataPacket,
    MarkerParket,
    LiteralDataPacket,
    TrustPacket,
    UserIdPacket((StartPosition, String)),
    PublicSubkeyPacket,
    UserAttributePacket,
    SymEncryptedIntegrityProtectedDataPacket,
    ReservedModif,
    Reserved,
    PaddingPacket,
    UnknownPacket,
}

impl InnerPacket {
    /// parse from tag id
    fn parse_from_tag(stream: &mut Stream<'_>, tag: u8, length: usize) -> Result<Self, String> {
        let start_pos = stream.pos;
        let res = match tag {
            0 => Self::ReservedReally,
            1 => Self::PublicKeyEncryptedSessionKeyPacket,
            2 => Self::SignaturePacket,
            3 => Self::SymmetricKeyEncryptedSessionKeyPacket,
            4 => Self::OnePassSignaturePacket,
            5 => Self::SecretKeyPacket,
            6 => {
                let version = stream.octet().ok_or("Cannot get version")?;
                if version == 6 || version == 4 || version == 3 {
                    let created_at = stream.time().ok_or("Cannot get creation date")?;
                    let valid_days = if version == 3 { stream.uint16() } else { None };
                    let algorithm = stream.octet().ok_or("Cannot get algo")?;
                    let octet_count = if version == 6 { stream.uint32() } else { None };
                    let key = PublicKey::parse_from_tag(stream, algorithm);
                    let public_key_packet = PublicKeyPacket {
                        version,
                        created_at,
                        valid_days,
                        algorithm,
                        octet_count,
                        key,
                    };
                    Self::PublicKeyPacket((start_pos, public_key_packet))
                } else {
                    return Err("Unsupported version".to_string());
                }
            }
            7 => Self::SecretSubKeyPacket,
            8 => Self::CompressedDataPacket,
            9 => Self::SymmetricallyEncryptedDataPacket,
            10 => Self::MarkerParket,
            11 => Self::LiteralDataPacket,
            12 => Self::TrustPacket,
            13 => {
                let user_id = stream.utf8(length).ok_or("Cannot get string")?;
                Self::UserIdPacket((start_pos, user_id))
            }
            14 => Self::PublicSubkeyPacket,
            17 => Self::UserAttributePacket,
            18 => Self::SymEncryptedIntegrityProtectedDataPacket,
            19 => Self::ReservedModif,
            20 => Self::Reserved,
            21 => Self::PaddingPacket,
            _ => Self::UnknownPacket,
        };
        Ok(res)
    }

    /// Get the name
    fn name(&self) -> &str {
        match self {
            Self::ReservedReally => "Reserved - a packet tag MUST NOT have this value",
            Self::PublicKeyEncryptedSessionKeyPacket => "Public Key Encrypted Session Key Packet",
            Self::SignaturePacket => "Signature Packet",
            Self::SymmetricKeyEncryptedSessionKeyPacket => {
                "Symmetric-Key Encrypted Session Key Packet"
            }
            Self::OnePassSignaturePacket => "One-Pass Signature Packet",
            Self::SecretKeyPacket => "Secret Key Packet",
            Self::PublicKeyPacket(_) => "Public Key Packet",
            Self::SecretSubKeyPacket => "Secret Subkey Packet",
            Self::CompressedDataPacket => "Compressed Data Packet",
            Self::SymmetricallyEncryptedDataPacket => "Symmetrically Encrypted Data Packet",
            Self::MarkerParket => "Marker Packet",
            Self::LiteralDataPacket => "Literal Data Packet",
            Self::TrustPacket => "Trust Packet",
            Self::UserIdPacket(_) => "User ID Packet",
            Self::PublicSubkeyPacket => "Public Subkey Packet",
            Self::UserAttributePacket => "User Attribute Packet",
            Self::SymEncryptedIntegrityProtectedDataPacket => {
                "Sym. Encrypted and Integrity Protected Data Packet"
            }
            Self::ReservedModif => "Reserved (formerly Modification Detection Code Packet)",
            Self::Reserved => "Reserved",
            Self::PaddingPacket => "Padding Packet",
            Self::UnknownPacket => "Unknown packet",
        }
    }

    /// Show ui
    fn ui(&self, ui: &mut egui::Ui) -> Option<RangeInclusive<usize>> {
        let mut range = None;
        match self {
            Self::PublicKeyPacket((pos, public_key)) => {
                let PublicKeyPacket {
                    version,
                    created_at,
                    valid_days,
                    algorithm,
                    octet_count,
                    key,
                } = public_key;
                ui.horizontal(|ui| {
                    ui.label(format!("Version: {version}"));
                    ui.label(format!("Created at: {created_at:?}"));
                    if let Some(v) = valid_days {
                        ui.label(format!("Valid days: {v:?}"));
                    }
                    ui.label(format!("Algorithm: {algorithm:?}"));
                    if let Some(oc) = octet_count {
                        ui.label(format!("Octed count: {oc:?}"));
                    }
                    key.ui(ui);
                });
            }
            Self::UserIdPacket((pos, user_id)) => {
                ui.horizontal(|ui| {
                    ui.label("userId");
                    ui.label(user_id);
                    if ui.button("Show").clicked() {
                        let end = pos.saturating_add(user_id.len()).saturating_sub(1);
                        range = Some(*pos..=end);
                    }
                });
            }
            Self::UnknownPacket => {
                ui.label("Unknown packet");
            }
            _ => {
                ui.label("TODO");
            }
        }
        range
    }
}

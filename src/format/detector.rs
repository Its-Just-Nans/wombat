//! File format reader

use crate::format::FileFormat;

impl FileFormat {
    /// Get from bytes
    pub(crate) fn from_bytes<B: AsRef<[u8]>>(all_bytes: B) -> Self {
        let all_bytes = all_bytes.as_ref();
        if all_bytes.is_empty() {
            return Self::Empty;
        }
        let max_len = Self::SIGNATURE_MAX_LEN.min(all_bytes.len());
        let Some(sign_part) = all_bytes.get(..max_len) else {
            return Self::Unknown;
        };

        if let Some(fmt) = Self::from_signature(sign_part) {
            return Self::from_fmt_reader(fmt, all_bytes).unwrap_or(Self::Unknown);
        }
        Self::Unknown
    }

    /// Dispatches to the appropriate format-specific reader based on the initial signature match.
    #[inline]
    pub(crate) fn from_fmt_reader(fmt: Self, buf: &[u8]) -> Result<Self, std::io::Error> {
        Ok(match fmt {
            Self::ExtensibleMarkupLanguage => Self::from_xml_reader(buf)?,
            Self::Zip => Self::from_zip_reader(buf)?,
            _ => fmt,
        })
    }

    /// Determines the file format from an XML reader.
    pub(crate) fn from_xml_reader(buf: &[u8]) -> Result<Self, std::io::Error> {
        use std::io::{Cursor, Read, Seek, SeekFrom};
        let mut reader = Cursor::new(buf);
        // Rewinds to the beginning of the stream plus the size of the XML file format signature.
        reader.seek(SeekFrom::Start(5))?;

        // Creates and fills a buffer.
        let mut buf = [0; 8192];
        let nread = reader.read(&mut buf)?;
        let buf = &buf[..nread];

        // Checks if the buffer holds markers indicating the presence of various file formats.
        Ok(if contains_slice(buf, b"<abiword template=\"false\"") {
            Self::Abiword
        } else if contains_slice(buf, b"<abiword template=\"true\"") {
            Self::AbiwordTemplate
        } else if contains_slice(buf, b"<amf") {
            Self::AdditiveManufacturingFormat
        } else if contains_slice(buf, b"<ASX") || contains_slice(buf, b"<asx") {
            Self::AdvancedStreamRedirector
        } else if contains_slice(buf, b"<feed") {
            Self::Atom
        } else if contains_slice(buf, b"<COLLADA") {
            Self::CollaborativeDesignActivity
        } else if contains_slice(buf, b"<mxfile") {
            Self::DrawIo
        } else if contains_slice(buf, b"<X3D") {
            Self::Extensible3d
        } else if contains_slice(buf, b"<xsl") {
            Self::ExtensibleStylesheetLanguageTransformations
        } else if contains_slice(buf, b"<FictionBook") {
            Self::Fictionbook
        } else if contains_slice(buf, b"<gml") {
            Self::GeographyMarkupLanguage
        } else if contains_slice(buf, b"<gpx") {
            Self::GpsExchangeFormat
        } else if contains_slice(buf, b"<kml") {
            Self::KeyholeMarkupLanguage
        } else if contains_slice(buf, b"<math") {
            Self::MathematicalMarkupLanguage
        } else if contains_slice(buf, b"<MPD") {
            Self::MpegDashMpd
        } else if contains_slice(buf, b"<score-partwise") {
            Self::Musicxml
        } else if contains_slice(buf, b"<rss") {
            Self::ReallySimpleSyndication
        } else if contains_slice(buf, b"<SVG") || contains_slice(buf, b"<svg") {
            Self::ScalableVectorGraphics
        } else if contains_slice(buf, b"<soap") {
            Self::SimpleObjectAccessProtocol
        } else if contains_slice(buf, b"<map") {
            Self::TiledMapXml
        } else if contains_slice(buf, b"<tileset") {
            Self::TiledTilesetXml
        } else if contains_slice(buf, b"<tt")
            && contains_slice(buf, b"xmlns=\"http://www.w3.org/ns/ttml\"")
        {
            Self::TimedTextMarkupLanguage
        } else if contains_slice(buf, b"<TrainingCenterDatabase") {
            Self::TrainingCenterXml
        } else if contains_slice(buf, b"<uof:UOF")
            && contains_slice(buf, b"uof:mimetype=\"vnd.uof.presentation\"")
        {
            Self::UniformOfficeFormatPresentation
        } else if contains_slice(buf, b"<uof:UOF")
            && contains_slice(buf, b"uof:mimetype=\"vnd.uof.spreadsheet\"")
        {
            Self::UniformOfficeFormatSpreadsheet
        } else if contains_slice(buf, b"<uof:UOF")
            && contains_slice(buf, b"uof:mimetype=\"vnd.uof.text\"")
        {
            Self::UniformOfficeFormatText
        } else if contains_slice(buf, b"<USFSubtitles") {
            Self::UniversalSubtitleFormat
        } else if contains_slice(buf, b"<xliff") {
            Self::XmlLocalizationInterchangeFileFormat
        } else if contains_slice(buf, b"<playlist") {
            Self::XmlShareablePlaylistFormat
        } else {
            Self::ExtensibleMarkupLanguage
        })
    }

    /// Determines the file format from a ZIP reader.
    #[allow(clippy::too_many_lines)]
    pub(crate) fn from_zip_reader(buf: &[u8]) -> Result<Self, std::io::Error> {
        use std::io::{BufReader, Cursor, Read, Seek, SeekFrom};
        use zip::unstable::LittleEndianReadExt;
        // Maximum number of entries that can be processed by the reader.
        const ENTRY_LIMIT: usize = 1024;

        // Signature of the ZIP64 end of central directory locator.
        const EOCD64_LOCATOR_SIGNATURE: &[u8] = b"PK\x06\x07";

        // Signature of the end of central directory record.
        const EOCD_SIGNATURE: &[u8] = b"PK\x05\x06";

        // Size of the ZIP64 end of central directory locator.
        const EOCD64_LOCATOR_SIZE: usize = 20;

        // Maximum size of the end of central directory record.
        const EOCD_MAX_SIZE: usize = EOCD_MIN_SIZE + u16::MAX as usize;

        // Minimum size of the end of central directory record.
        const EOCD_MIN_SIZE: usize = 22;

        let reader = Cursor::new(buf);

        // Creates a buffered reader.
        let mut reader = BufReader::new(reader);

        // Retrieves the stream length.
        let len = reader.seek(SeekFrom::End(0))?;

        // Searches for the end of central directory record.
        let offset = len.saturating_sub(EOCD_MAX_SIZE as u64);
        reader.seek(SeekFrom::Start(offset))?;
        let buf_index = {
            let size = (len as usize).clamp(EOCD_MIN_SIZE, EOCD_MAX_SIZE);
            let mut buf = vec![0; size];
            reader.read_exact(&mut buf)?;
            let needle = EOCD_SIGNATURE;
            let pos = buf.windows(needle.len()).rposition(|w| w == needle);
            pos.ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "cannot find the EOCD")
            })?
        };

        let eocd_offset = offset + buf_index as u64;

        // Checks for ZIP64 end of central directory locator.
        let mut zip64 = false;
        if eocd_offset as usize >= EOCD64_LOCATOR_SIZE {
            // Seeks to the ZIP64 end of central directory locator.
            reader.seek(SeekFrom::Start(eocd_offset - EOCD64_LOCATOR_SIZE as u64))?;

            // Reads the signature.
            let signature = {
                let mut buf = vec![0; 4];
                reader.read_exact(&mut buf)?;
                buf
            };

            // Checks the signature.
            if signature == EOCD64_LOCATOR_SIGNATURE {
                zip64 = true;
            }
        }

        // Reads the number of entries and the start of central directory offset.
        let (number_of_entries, socd_offset) = if zip64 {
            // Reads the offset of the ZIP64 end of central directory record.
            reader.seek(SeekFrom::Current(4))?;
            let eocd64_offset = reader.read_u64_le()?;

            // Reads the number of entries.
            reader.seek(SeekFrom::Start(eocd64_offset + 32))?;
            let number_of_entries = reader.read_u64_le()?;

            // Reads the start of central directory offset.
            reader.seek(SeekFrom::Current(8))?;
            let socd_offset = reader.read_u64_le()?;

            // Returns the result.
            (number_of_entries as usize, socd_offset)
        } else {
            // Reads the number of entries.
            reader.seek(SeekFrom::Start(eocd_offset + 10))?;
            let number_of_entries = reader.read_u16_le()?;

            // Reads the start of central directory offset.
            reader.seek(SeekFrom::Current(4))?;
            let socd_offset = reader.read_u32_le()?;

            // Returns the result.
            (number_of_entries as usize, u64::from(socd_offset))
        };

        // Seeks to the start of central directory.
        reader.seek(SeekFrom::Start(socd_offset))?;

        // Sets the default value.
        let mut fmt = Self::Zip;

        // Browses central directory headers.
        for _ in 0..std::cmp::min(ENTRY_LIMIT, number_of_entries) {
            // Reads the compressed size.
            reader.seek(SeekFrom::Current(20))?;
            let compressed_size = reader.read_u32_le()?;

            // Reads the uncompressed size.
            let uncompressed_size = reader.read_u32_le()?;

            // Reads the filename length.
            let filename_len = reader.read_u16_le()?;

            // Reads the extra field length.
            let extra_field_len = reader.read_u16_le()?;

            // Reads the file comment length.
            let file_comment_len = reader.read_u16_le()?;

            // Reads the relative offset of local file header.
            reader.seek(SeekFrom::Current(8))?;
            let offset = reader.read_u32_le()?;

            // Reads the filename.
            let filename = {
                let mut buf = vec![0; filename_len as usize];
                reader.read_exact(&mut buf)?;

                String::from_utf8_lossy(&buf).to_string()
            };

            // Checks the filename.
            match filename.as_str() {
                "AndroidManifest.xml" => return Ok(Self::AndroidPackage),
                "AppManifest.xaml" => return Ok(Self::Xap),
                "AppxManifest.xml" => return Ok(Self::WindowsAppPackage),
                "AppxMetadata/AppxBundleManifest.xml" => return Ok(Self::WindowsAppBundle),
                "BundleConfig.pb" => return Ok(Self::AndroidAppBundle),
                "DOMDocument.xml" => return Ok(Self::FlashCs5Project),
                "META-INF/AIR/application.xml" => return Ok(Self::AdobeIntegratedRuntime),
                "META-INF/MANIFEST.MF" => fmt = Self::JavaArchive,
                "META-INF/application.xml" => return Ok(Self::EnterpriseApplicationArchive),
                "META-INF/mozilla.rsa" => return Ok(Self::Xpinstall),
                "WEB-INF/web.xml" => return Ok(Self::WebApplicationArchive),
                "canvas.fig" => return Ok(Self::FigmaDesign),
                "doc.kml" => return Ok(Self::KeyholeMarkupLanguageZip),
                "document.json" => return Ok(Self::Sketch43),
                "extension.vsixmanifest" => return Ok(Self::MicrosoftVisualStudioExtension),
                "mimetype" if compressed_size == uncompressed_size => {
                    // Seeks to the filename of the local file header.
                    reader.seek(SeekFrom::Start(u64::from(offset) + 26))?;

                    // Reads the filename length.
                    let filename_len = reader.read_u16_le()?;

                    // Reads the extra field length.
                    let extra_field_len = reader.read_u16_le()?;

                    // Seeks to the data.
                    reader.seek(SeekFrom::Current(
                        i64::from(filename_len) + i64::from(extra_field_len),
                    ))?;

                    // Reads the data.
                    let data = {
                        let mut buf = vec![0; compressed_size as usize];
                        reader.read_exact(&mut buf)?;

                        String::from_utf8_lossy(&buf).to_string()
                    };

                    // Checks the trimmed data.
                    return Ok(match data.trim() {
                        "application/epub+zip" => Self::ElectronicPublication,
                        "application/vnd.adobe.indesign-idml-package" => {
                            Self::IndesignMarkupLanguage
                        }
                        "application/vnd.oasis.opendocument.base"
                        | "application/vnd.oasis.opendocument.database" => {
                            Self::OpendocumentDatabase
                        }
                        "application/vnd.oasis.opendocument.formula" => Self::OpendocumentFormula,
                        "application/vnd.oasis.opendocument.formula-template" => {
                            Self::OpendocumentFormulaTemplate
                        }
                        "application/vnd.oasis.opendocument.graphics" => Self::OpendocumentGraphics,
                        "application/vnd.oasis.opendocument.graphics-template" => {
                            Self::OpendocumentGraphicsTemplate
                        }
                        "application/vnd.oasis.opendocument.presentation" => {
                            Self::OpendocumentPresentation
                        }
                        "application/vnd.oasis.opendocument.presentation-template" => {
                            Self::OpendocumentPresentationTemplate
                        }
                        "application/vnd.oasis.opendocument.spreadsheet" => {
                            Self::OpendocumentSpreadsheet
                        }
                        "application/vnd.oasis.opendocument.spreadsheet-template" => {
                            Self::OpendocumentSpreadsheetTemplate
                        }
                        "application/vnd.oasis.opendocument.text" => Self::OpendocumentText,
                        "application/vnd.oasis.opendocument.text-master" => {
                            Self::OpendocumentTextMaster
                        }
                        "application/vnd.oasis.opendocument.text-master-template" => {
                            Self::OpendocumentTextMasterTemplate
                        }
                        "application/vnd.oasis.opendocument.text-template" => {
                            Self::OpendocumentTextTemplate
                        }
                        "application/vnd.recordare.musicxml" => Self::MusicxmlZip,
                        "application/vnd.sun.xml.calc" => Self::SunXmlCalc,
                        "application/vnd.sun.xml.calc.template" => Self::SunXmlCalcTemplate,
                        "application/vnd.sun.xml.draw" => Self::SunXmlDraw,
                        "application/vnd.sun.xml.draw.template" => Self::SunXmlDrawTemplate,
                        "application/vnd.sun.xml.impress" => Self::SunXmlImpress,
                        "application/vnd.sun.xml.impress.template" => Self::SunXmlImpressTemplate,
                        "application/vnd.sun.xml.math" => Self::SunXmlMath,
                        "application/vnd.sun.xml.writer" => Self::SunXmlWriter,
                        "application/vnd.sun.xml.writer.global" => Self::SunXmlWriterGlobal,
                        "application/vnd.sun.xml.writer.template" => Self::SunXmlWriterTemplate,
                        "image/openraster" => Self::Openraster,
                        _ => Self::Zip,
                    });
                }
                _ => {
                    if filename.starts_with("Fusion[Active]/") {
                        return Ok(Self::Autodesk123d);
                    } else if filename.starts_with("circuitdiagram/") {
                        return Ok(Self::CircuitDiagramDocument);
                    } else if filename.starts_with("dwf/") {
                        return Ok(Self::DesignWebFormatXps);
                    } else if filename.ends_with(".fb2") && !filename.contains('/') {
                        return Ok(Self::FictionbookZip);
                    } else if filename.starts_with("FusionAssetName[Active]/") {
                        return Ok(Self::Fusion360);
                    } else if filename.starts_with("Payload/") && filename.contains(".app/") {
                        return Ok(Self::IosAppStorePackage);
                    } else if filename.starts_with("word/") {
                        return Ok(Self::OfficeOpenXmlDocument);
                    } else if filename.starts_with("visio/") {
                        return Ok(Self::OfficeOpenXmlDrawing);
                    } else if filename.starts_with("ppt/") {
                        return Ok(Self::OfficeOpenXmlPresentation);
                    } else if filename.starts_with("xl/") {
                        return Ok(Self::OfficeOpenXmlSpreadsheet);
                    } else if filename.starts_with("Documents/") && filename.ends_with(".fpage") {
                        return Ok(Self::Openxps);
                    } else if filename.starts_with("SpaceClaim/") {
                        return Ok(Self::SpaceclaimDocument);
                    } else if filename.starts_with("3D/") && filename.ends_with(".model") {
                        return Ok(Self::ThreeDimensionalManufacturingFormat);
                    } else if (filename.ends_with(".usd")
                        || filename.ends_with(".usda")
                        || filename.ends_with(".usdc"))
                        && !filename.contains('/')
                    {
                        return Ok(Self::UniversalSceneDescriptionZip);
                    }
                }
            }

            // Seeks to the next central directory entry.
            reader.seek(SeekFrom::Current(
                i64::from(extra_field_len) + i64::from(file_comment_len),
            ))?;
        }
        Ok(fmt)
    }
}

fn contains_slice<T: PartialEq>(haystack: &[T], needle: &[T]) -> bool {
    needle.is_empty() || haystack.windows(needle.len()).any(|w| w == needle)
}

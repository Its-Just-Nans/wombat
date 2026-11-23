//! File format

pub(crate) mod detector;

/// File kind
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum FileKind {
    /// Files and directories stored in a single, possibly compressed, archive.
    Archive,
    /// Musics, sound effects, and spoken audio recordings.
    Audio,
    /// Compressed single files or streams.
    Compressed,
    /// Organized collections of data.
    Database,
    /// Visual information using graphics and spatial relationships.
    Diagram,
    /// Floppy disk images, optical disc images and virtual machine disks.
    Disk,
    /// Word processing and desktop publishing documents.
    Document,
    /// Electronic books.
    Ebook,
    /// Machine-executable code, virtual machine code and shared libraries.
    Executable,
    /// Typefaces used for displaying text on screen or in print.
    Font,
    /// Mathematical formulas.
    Formula,
    /// Collections of geospatial features, GPS tracks and other location-related files.
    Geospatial,
    /// Animated images, icons, cursors, raster graphics and vector graphics.
    Image,
    /// Data that provides information about other data.
    Metadata,
    /// 3D models, CAD drawings, and other types of files used for creating or displaying 3D images.
    Model,
    /// Data which do not fit in any of the other kinds.
    Other,
    /// Collections of files bundled together for software distribution.
    Package,
    /// Lists of audio or video files, organized in a specific order for sequential playback.
    Playlist,
    /// Slide shows.
    Presentation,
    /// Copies of a read-only memory chip of computers, cartridges, or other electronic devices.
    Rom,
    /// Data in tabular form.
    Spreadsheet,
    /// Subtitles and captions.
    Subtitle,
    /// Moving images, possibly with color and coordinated sound.
    Video,
}

impl core::fmt::Display for FileKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// declare the format
macro_rules! define_format {
    (
        $(
            format = $format:ident
            signatures = [
                $(signature = $($signature:literal $(offset = $offset:literal)?),+)*
            ]
            name = $name:literal
            short_name = $short_name:literal
            mime = $mime:literal
            extensions = [$($extension:expr),* $(,)?]
            kind = $kind:ident
        )*
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[derive(serde::Serialize, serde::Deserialize)]
        #[allow(clippy::enum_variant_names)]
        pub enum FileFormat {
            $($format,)*
        }

        impl FileFormat {
            pub(crate) const SIGNATURE_MAX_LEN: usize = {
                let sizes: &[usize] = &[
                    $($($(
                            $($offset +)? $signature.len(),
                    )*)*)*
                ];

                let mut max = 0;
                let mut i = 0;

                while i < sizes.len() {
                    if sizes[i] > max {
                        max = sizes[i];
                    }
                    i += 1;
                }
                max
            };

            #[allow(clippy::int_plus_one)]
            pub(crate) fn from_signature(bytes: &[u8]) -> Option<Self> {
                $(
                    if false $(
                        || (true $(
                            && bytes.len() >= $($offset +)? $signature.len()
                            && &bytes[$($offset)?..$($offset +)? $signature.len()] == $signature
                        )+)
                    )* {
                        return Some(Self::$format);
                    }
                )*
                None
            }

            pub const fn name(&self) -> &str {
                match self {
                    $(
                        Self::$format => $name,
                    )*
                }
            }

            pub const fn kind(&self) -> FileKind {
                match self {
                    $(
                        Self::$format => FileKind::$kind,
                    )*
                }
            }

            pub const fn mime(&self) -> &str {
                match self {
                    $(
                        Self::$format => $mime,
                    )*
                }
            }

            // pub const f: signatures(&self) -> &'static [&'static [u8]] {
            //     match self {
            //         $(
            //             Self::$format => &[$($signature,)*],
            //         )*
            //     }
            // }

            pub const fn extensions(&self) -> &'static [&'static str] {
                match self {
                    $(
                        Self::$format => &[$($extension,)*],
                    )*
                }
            }

            pub const fn extension(&self) -> &str {
                self.extensions()[0]
            }
        }


        // pub const FORMATS: &[FileFormat] = &[
        //     $(
        //         FileFormat::$format,
        //     )*
        // ];
    };
}

// impl FileFormat {
//         use std::io::Cursor;
//         use std::io::Read;
//         let reader = Cursor::new(bytes);
//         let mut buf = vec![0u8; Self::SIGNATURE_MAX_LEN];
//         let mut nread = 0;
//         while nread < buf.len() {
//             match reader.read(&mut buf[nread..])? {
//                 0 => break,
//                 n => nread += n,
//             }
//         }
//
//         let f = if nread == 0 {
//             Self::Empty
//         } else if let Some(fmt) = Self::from_signature(&buf[..nread]) {
//             Self::from_fmt_reader(fmt, &mut reader)
//                 .unwrap_or_else(|_| Self::from_generic_reader(&mut reader))
//         } else {
//             Self::from_generic_reader(&mut reader)
//         };
//         Ok(f)
//     }
// }

define_format! {
    format = Abiword
    signatures = [
        signature = b"\xEF\xBB\xBF<!DOCTYPE abiword PUBLIC", b"<abiword template=\"false\"" offset = 102
        signature = b"<!DOCTYPE abiword PUBLIC", b"<abiword template=\"false\"" offset = 102
        signature = b"\xEF\xBB\xBF<abiword template=\"false\""
        signature = b"<abiword template=\"false\""
    ]
    name = "AbiWord"
    short_name = "AbiWord"
    mime = "application/x-abiword"
    extensions = [
        "abw",
    ]
    kind = Document

    format = AbiwordTemplate
    signatures = [
        signature = b"\xEF\xBB\xBF<!DOCTYPE abiword PUBLIC", b"<abiword template=\"true\"" offset = 102
        signature = b"<!DOCTYPE abiword PUBLIC", b"<abiword template=\"true\"" offset = 102
        signature = b"\xEF\xBB\xBF<abiword template=\"true\""
        signature = b"<abiword template=\"true\""
    ]
    name = "AbiWord Template"
    short_name = "AbiWord Template"
    mime = "application/x-abiword-template"
    extensions = [
        "awt",
    ]
    kind = Document

    format = Ace
    signatures = [
        signature = b"**ACE**" offset = 7
    ]
    name = "ACE"
    short_name = "ACE"
    mime = "application/x-ace-compressed"
    extensions = [
        "ace",
    ]
    kind = Archive

    format = ActionsMediaVideo
    signatures = [
        signature = b"RIFF", b"AMV " offset = 8
    ]
    name = "Actions Media Video"
    short_name = "Actions Media Video"
    mime = "video/x-amv"
    extensions = [
        "amv",
    ]
    kind = Video

    format = Activemime
    signatures = [
        signature = b"ActiveMime\0\0"
    ]
    name = "ActiveMime"
    short_name = "ActiveMime"
    mime = "application/x-mso"
    extensions = [
        "mso",
    ]
    kind = Other

    format = AdaptableScalableTextureCompression
    signatures = [
        signature = b"\x13\xAB\xA1\x5C"
    ]
    name = "Adaptable Scalable Texture Compression"
    short_name = "Adaptable Scalable Texture Compression"
    mime = "image/x-astc"
    extensions = [
        "astc",
    ]
    kind = Image

    format = AdaptiveMultiRate
    signatures = [
       signature = b"#!AMR"
    ]
    name = "Adaptive Multi-Rate"
    short_name = "Adaptive Multi-Rate"
    mime = "audio/amr"
    extensions = [
        "amr",
    ]
    kind = Audio

    format = AdditiveManufacturingFormat
    signatures = [
        signature = b"\xEF\xBB\xBF<amf"
        signature = b"<amf"
    ]
    name = "Additive Manufacturing Format"
    short_name = "Additive Manufacturing Format"
    mime = "application/x-amf"
    extensions = [
        "amf",
    ]
    kind = Model

    format = AdobeIllustratorArtwork
    signatures = [
    ]
    name = "Adobe Illustrator Artwork"
    short_name = "Adobe Illustrator Artwork"
    mime = "application/vnd.adobe.illustrator"
    extensions = [
        "ai",
    ]
    kind = Image

    format = AdobeIndesignDocument
    signatures = [
        signature = b"\x06\x06\xED\xF5\xD8\x1D\x46\xE5\xBD\x31\xEF\xE7\xFE\x74\xB7\x1D"
    ]
    name = "Adobe InDesign Document"
    short_name = "Adobe InDesign Document"
    mime = "application/x-indesign"
    extensions = [
        "indd",
    ]
    kind = Document

    format = AdobeIntegratedRuntime
    signatures = []
    name = "Adobe Integrated Runtime"
    short_name = "Adobe Integrated Runtime"
    mime = "application/vnd.adobe.air-application-installer-package+zip"
    extensions = [
        "air",
    ]
    kind = Package

    format = AdobePhotoshopDocument
    signatures = [
        signature = b"8BPS"
    ]
    name = "Adobe Photoshop Document"
    short_name = "Adobe Photoshop Document"
    mime = "image/vnd.adobe.photoshop"
    extensions = [
        "psd",
    ]
    kind = Image

    format = AdvancedAudioCoding
    signatures = [
        signature = b"\xFF\xF1"
        signature = b"\xFF\xF9"
    ]
    name = "Advanced Audio Coding"
    short_name = "Advanced Audio Coding"
    mime = "audio/aac"
    extensions = [
        "aac",
    ]
    kind = Audio

    format = AdvancedStreamRedirector
    signatures = [
        signature = b"\xEF\xBB\xBF<ASX"
        signature = b"\xEF\xBB\xBF<asx"
        signature = b"<ASX"
        signature = b"<asx"
    ]
    name = "Advanced Stream Redirector"
    short_name = "Advanced Stream Redirector"
    mime = "video/x-ms-asx"
    extensions = [
        "asx",
    ]
    kind = Playlist

    format = AdvancedSystemsFormat
    signatures = [
        signature = b"\x30\x26\xB2\x75\x8E\x66\xCF\x11\xA6\xD9\x00\xAA\x00\x62\xCE\x6C"
    ]
    name = "Advanced Systems Format"
    short_name = "Advanced Systems Format"
    mime = "application/vnd.ms-asf"
    extensions = [
        "asf",
    ]
    kind = Other

    format = AgeEncryption
    signatures = [
        signature = b"age-encryption.org/"
    ]
    name = "age encryption"
    short_name = "age encryption"
    mime = "application/x-age-encryption"
    extensions = [
        "age",
    ]
    kind = Other

    format = Alz
    signatures = [
        signature = b"ALZ\x01"
    ]
    name = "ALZ"
    short_name = "ALZ"
    mime = "application/x-alz-compressed"
    extensions = [
        "alz",
    ]
    kind = Archive

    format = AmigaDiskFile
    signatures = [
        signature = b"DOS\0"
        signature = b"DOS\x01"
        signature = b"DOS\x02"
        signature = b"DOS\x03"
        signature = b"DOS\x04"
        signature = b"DOS\x05"
    ]
    name = "Amiga Disk File"
    short_name = "Amiga Disk File"
    mime = "application/x-amiga-disk-format"
    extensions = [
        "adf",
    ]
    kind = Disk

    format = AndroidAppBundle
    signatures = []
    name = "Android App Bundle"
    short_name = "Android App Bundle"
    mime = "application/vnd.android.aab"
    extensions = [
        "aab",
    ]
    kind = Package

    format = AndroidBinaryXml
    signatures = [
        signature = b"\x03\x00\x08\x00"
    ]
    name = "Android Binary XML"
    short_name = "Android Binary XML"
    mime = "application/vnd.android.axml"
    extensions = [
        "xml",
    ]
    kind = Metadata

    format = AndroidPackage
    signatures = []
    name = "Android Package"
    short_name = "Android Package"
    mime = "application/vnd.android.package-archive"
    extensions = [
        "apk",
    ]
    kind = Package

    format = AndroidResourceStorageContainer
    signatures = [
        signature = b"\x02\x00\x0C\x00"
    ]
    name = "Android Resource Storage Container"
    short_name = "Android Resource Storage Container"
    mime = "application/vnd.android.arsc"
    extensions = [
        "arsc",
    ]
    kind = Other

    format = AnimatedPortableNetworkGraphics
    signatures = [
        signature = b"\x89PNG\r\n\x1A\n", b"acTL" offset = 37
    ]
    name = "Animated Portable Network Graphics"
    short_name = "Animated Portable Network Graphics"
    mime = "image/apng"
    extensions = [
        "apng",
    ]
    kind = Image

    format = ApacheArrowColumnar
    signatures = [
        signature = b"ARROW1"
    ]
    name = "Apache Arrow Columnar"
    short_name = "Apache Arrow Columnar"
    mime = "application/vnd.apache.arrow.file"
    extensions = [
        "arrow",
    ]
    kind = Other

    format = ApacheAvro
    signatures = [
        signature = b"Obj\x01"
    ]
    name = "Apache Avro"
    short_name = "Apache Avro"
    mime = "application/vnd.apache.avro"
    extensions = [
        "avro",
    ]
    kind = Other

    format = ApacheParquet
    signatures = [
        signature = b"PAR1"
    ]
    name = "Apache Parquet"
    short_name = "Apache Parquet"
    mime = "application/vnd.apache.parquet"
    extensions = [
        "parquet",
    ]
    kind = Other

    format = Appimage
    signatures = [
        signature = b"\x7FELF", b"AI\x02\x00\x00\x00\x00\x00" offset = 8
    ]
    name = "AppImage"
    short_name = "AppImage"
    mime = "application/x-appimage"
    extensions = [
        "AppImage",
    ]
    kind = Package

    format = AppleDiskImage
    signatures = [
        signature = b"\x78\x01"
    ]
    name = "Apple Disk Image"
    short_name = "Apple Disk Image"
    mime = "application/x-apple-diskimage"
    extensions = [
        "dmg",
    ]
    kind = Disk

    format = AppleIconImage
    signatures = [
        signature = b"icns"
    ]
    name = "Apple Icon Image"
    short_name = "Apple Icon Image"
    mime = "image/x-icns"
    extensions = [
        "icns",
    ]
    kind = Image

    format = AppleItunesAudio
    signatures = [
        signature = b"ftypM4A" offset = 4
    ]
    name = "Apple iTunes Audio"
    short_name = "Apple iTunes Audio"
    mime = "audio/x-m4a"
    extensions = [
        "m4a",
    ]
    kind = Audio

    format = AppleItunesAudiobook
    signatures = [
        signature = b"ftypM4B" offset = 4
    ]
    name = "Apple iTunes Audiobook"
    short_name = "Apple iTunes Audiobook"
    mime = "audio/mp4"
    extensions = [
        "m4b",
    ]
    kind = Audio

    format = AppleItunesProtectedAudio
    signatures = [
        signature = b"ftypM4P" offset = 4
    ]
    name = "Apple iTunes Protected Audio"
    short_name = "Apple iTunes Protected Audio"
    mime = "audio/mp4"
    extensions = [
        "m4p",
    ]
    kind = Audio

    format = AppleItunesVideo
    signatures = [
        signature = b"ftypM4V" offset = 4
    ]
    name = "Apple iTunes Video"
    short_name = "Apple iTunes Video"
    mime = "video/x-m4v"
    extensions = [
        "m4v",
    ]
    kind = Video

    format = AppleQuicktime
    signatures = [
        signature = b"ftypqt" offset = 4
        signature = b"free" offset = 4
        signature = b"mdat" offset = 4
        signature = b"moov" offset = 4
        signature = b"skip" offset = 4
        signature = b"wide" offset = 4
    ]
    name = "Apple QuickTime"
    short_name = "Apple QuickTime"
    mime = "video/quicktime"
    extensions = [
        "mov",
    ]
    kind = Video

    format = ArbitraryBinaryData
    signatures = []
    name = "Arbitrary Binary Data"
    short_name = "Arbitrary Binary Data"
    mime = "application/octet-stream"
    extensions = [
        "bin",
    ]
    kind = Other

    format = ArchivedByRobertJung
    signatures = [
        signature = b"\x60\xEA", b"\x02" offset = 10
    ]
    name = "Archived by Robert Jung"
    short_name = "Archived by Robert Jung"
    mime = "application/x-arj"
    extensions = [
        "arj",
    ]
    kind = Archive

    format = Atari7800Rom
    signatures = [
        signature = b"\x01ATARI7800"
    ]
    name = "Atari 7800 ROM"
    short_name = "Atari 7800 ROM"
    mime = "application/x-atari-7800-rom"
    extensions = [
        "a78",
    ]
    kind = Rom

    format = Atom
    signatures = [
        signature = b"\xEF\xBB\xBF<feed"
        signature = b"<feed"
    ]
    name = "Atom"
    short_name = "Atom"
    mime = "application/atom+xml"
    extensions = [
        "atom",
    ]
    kind = Other

    format = Au
    signatures = [
        signature = b".snd"
    ]
    name = "Au"
    short_name = "Au"
    mime = "audio/basic"
    extensions = [
        "au",
    ]
    kind = Audio

    format = AudioCodec3
    signatures = [
        signature = b"\x0B\x77"
    ]
    name = "Audio Codec 3"
    short_name = "Audio Codec 3"
    mime = "audio/ac3"
    extensions = [
        "ac3",
    ]
    kind = Audio

    format = AudioInterchangeFileFormat
    signatures = [
        signature = b"FORM", b"AIFC" offset = 8
        signature = b"FORM", b"AIFF" offset = 8
    ]
    name = "Audio Interchange File Format"
    short_name = "Audio Interchange File Format"
    mime = "audio/x-aiff"
    extensions = [
        "aiff",
    ]
    kind = Audio

    format = AudioVideoInterleave
    signatures = [
        signature = b"RIFF", b"AVI " offset = 8
    ]
    name = "Audio Video Interleave"
    short_name = "Audio Video Interleave"
    mime = "video/x-msvideo"
    extensions = [
        "avi",
    ]
    kind = Video

    format = AudioVisualResearch
    signatures = [
        signature = b"2BIT"
    ]
    name = "Audio Visual Research"
    short_name = "Audio Visual Research"
    mime = "audio/x-avr"
    extensions = [
        "avr",
    ]
    kind = Audio

    format = AutocadDrawing
    signatures = [
        signature = b"AC1."
        signature = b"AC10"
        signature = b"AC2."
        signature = b"MC0."
    ]
    name = "AutoCAD Drawing"
    short_name = "AutoCAD Drawing"
    mime = "application/x-dwg"
    extensions = [
        "dwg",
    ]
    kind = Model

    format = Autodesk123d
    signatures = []
    name = "Autodesk 123D"
    short_name = "Autodesk 123D"
    mime = "model/x-123dx"
    extensions = [
        "123dx",
    ]
    kind = Model

    format = AutodeskAlias
    signatures = [
        signature = b"\x8FStudioPacketFile"
    ]
    name = "Autodesk Alias"
    short_name = "Autodesk Alias"
    mime = "model/x-wire"
    extensions = [
        "wire",
    ]
    kind = Model

    format = AutodeskAnimator
    signatures = [
        signature = b"\x11\xAF" offset = 4
    ]
    name = "Autodesk Animator"
    short_name = "Autodesk Animator"
    mime = "video/x-fli"
    extensions = [
        "fli",
    ]
    kind = Video

    format = AutodeskAnimatorPro
    signatures = [
        signature = b"\x12\xAF" offset = 4
    ]
    name = "Autodesk Animator Pro"
    short_name = "Autodesk Animator Pro"
    mime = "video/x-flc"
    extensions = [
        "flc",
    ]
    kind = Video

    format = AutodeskInventorAssembly
    signatures = []
    name = "Autodesk Inventor Assembly"
    short_name = "Autodesk Inventor Assembly"
    mime = "model/x-iam"
    extensions = [
        "iam",
    ]
    kind = Model

    format = AutodeskInventorDrawing
    signatures = []
    name = "Autodesk Inventor Drawing"
    short_name = "Autodesk Inventor Drawing"
    mime = "model/x-idw"
    extensions = [
        "idw",
    ]
    kind = Model

    format = AutodeskInventorPart
    signatures = []
    name = "Autodesk Inventor Part"
    short_name = "Autodesk Inventor Part"
    mime = "model/x-ipt"
    extensions = [
        "ipt",
    ]
    kind = Model

    format = AutodeskInventorPresentation
    signatures = []
    name = "Autodesk Inventor Presentation"
    short_name = "Autodesk Inventor Presentation"
    mime = "model/x-ipn"
    extensions = [
        "ipn",
    ]
    kind = Model

    format = Av1ImageFileFormat
    signatures = [
        signature = b"ftypavif" offset = 4
    ]
    name = "AV1 Image File Format"
    short_name = "AV1 Image File Format"
    mime = "image/avif"
    extensions = [
        "avif",
    ]
    kind = Image

    format = Av1ImageFileFormatSequence
    signatures = [
        signature = b"ftypavis" offset = 4
    ]
    name = "AV1 Image File Format Sequence"
    short_name = "AV1 Image File Format Sequence"
    mime = "image/avif-sequence"
    extensions = [
        "avifs",
    ]
    kind = Image

    format = BdavMpeg2TransportStream
    signatures = [
        signature = b"\x47" offset = 4, b"\x47" offset = 196
    ]
    name = "BDAV MPEG-2 Transport Stream"
    short_name = "BDAV MPEG-2 Transport Stream"
    mime = "video/mp2t"
    extensions = [
        "m2ts",
    ]
    kind = Video

    format = BetterPortableGraphics
    signatures = [
        signature = b"BPG\xFB"
    ]
    name = "Better Portable Graphics"
    short_name = "Better Portable Graphics"
    mime = "image/bpg"
    extensions = [
        "bpg",
    ]
    kind = Image

    format = Bittorrent
    signatures = [
        signature = b"d8:announce"
    ]
    name = "BitTorrent"
    short_name = "BitTorrent"
    mime = "application/x-bittorrent"
    extensions = [
        "torrent",
    ]
    kind = Metadata

    format = Blender
    signatures = [
        signature = b"BLENDER"
    ]
    name = "Blender"
    short_name = "Blender"
    mime = "application/x-blender"
    extensions = [
        "blend",
    ]
    kind = Model

    format = BmfontAscii
    signatures = [
        signature = b"info face="
    ]
    name = "BMFont ASCII"
    short_name = "BMFont ASCII"
    mime = "application/x-angelcode-bmfont"
    extensions = [
        "fnt",
    ]
    kind = Font

    format = BmfontBinary
    signatures = [
        signature = b"BMF\x03"
    ]
    name = "BMFont Binary"
    short_name = "BMFont Binary"
    mime = "application/x-angelcode-bmfont"
    extensions = [
        "fnt",
    ]
    kind = Font

    format = BroadBandEbook
    signatures = [
        signature = b"L\0R\0F\0\0\0"
    ]
    name = "Broad Band eBook"
    short_name = "Broad Band eBook"
    mime = "application/x-lrf"
    extensions = [
        "lrf",
    ]
    kind = Ebook

    format = Bzip
    signatures = [
        signature = b"BZ0"
    ]
    name = "bzip"
    short_name = "bzip"
    mime = "application/x-bzip"
    extensions = [
        "bz",
    ]
    kind = Compressed

    format = Bzip2
    signatures = [
        signature = b"BZh"
    ]
    name = "bzip2"
    short_name = "bzip2"
    mime = "application/x-bzip2"
    extensions = [
        "bz2",
    ]
    kind = Compressed

    format = Bzip3
    signatures = [
        signature = b"BZ3v1"
    ]
    name = "BZip3"
    short_name = "BZip3"
    mime = "application/x-bzip3"
    extensions = [
        "bz3",
    ]
    kind = Compressed

    format = Cabinet
    signatures = [
        signature = b"ISc("
        signature = b"MSCF"
    ]
    name = "Cabinet"
    short_name = "Cabinet"
    mime = "application/vnd.ms-cab-compressed"
    extensions = [
        "cab",
    ]
    kind = Archive

    format = CanonRaw
    signatures = [
        signature = b"II\x1A\0\0\0HEAPCCDR"
    ]
    name = "Canon Raw"
    short_name = "Canon Raw"
    mime = "image/x-canon-crw"
    extensions = [
        "crw",
    ]
    kind = Image

    format = CanonRaw2
    signatures = [
        signature = b"\x49\x49\x2A\x00", b"CR" offset = 8
        signature = b"\x4D\x4D\x00\x2A", b"CR" offset = 8
    ]
    name = "Canon Raw 2"
    short_name = "Canon Raw 2"
    mime = "image/x-canon-cr2"
    extensions = [
        "cr2",
    ]
    kind = Image

    format = CanonRaw3
    signatures = [
        signature = b"ftypcrx" offset = 4
    ]
    name = "Canon Raw 3"
    short_name = "Canon Raw 3"
    mime = "image/x-canon-cr3"
    extensions = [
        "cr3",
    ]
    kind = Image

    format = CdAudio
    signatures = [
        signature = b"RIFF", b"CDDAfmt " offset = 8
    ]
    name = "CD Audio"
    short_name = "CD Audio"
    mime = "application/x-cdf"
    extensions = [
        "cda",
    ]
    kind = Metadata

    format = Cinema4d
    signatures = [
        signature = b"C4DC4D6" offset = 1
    ]
    name = "Cinema 4D"
    short_name = "Cinema 4D"
    mime = "model/x-c4d"
    extensions = [
        "c4d",
    ]
    kind = Model

    format = Cineon
    signatures = [
        signature = b"\x80\x2A\x5F\xD7"
    ]
    name = "Cineon"
    short_name = "Cineon"
    mime = "image/cineon"
    extensions = [
        "cin",
    ]
    kind = Image

    format = CircuitDiagramDocument
    signatures = []
    name = "Circuit Diagram Document"
    short_name = "Circuit Diagram Document"
    mime = "application/vnd.circuitdiagram.document.main+xml"
    extensions = [
        "cddx",
    ]
    kind = Diagram

    format = ClojureScript
    signatures = [
        signature = b"#!/usr/local/bin/clojure"
        signature = b"#!/usr/bin/env clojure"
        signature = b"#!/usr/local/bin/clj"
        signature = b"#!/usr/bin/clojure"
        signature = b"#!/usr/bin/env clj"
        signature = b"#!/usr/bin/clj"
    ]
    name = "Clojure Script"
    short_name = "Clojure Script"
    mime = "text/x-clojure"
    extensions = [
        "clj",
    ]
    kind = Other

    format = CollaborativeDesignActivity
    signatures = [
        signature = b"\xEF\xBB\xBF<COLLADA"
        signature = b"\xEF\xBB\xBF<collada"
        signature = b"<COLLADA"
        signature = b"<collada"
    ]
    name = "Collaborative Design Activity"
    short_name = "Collaborative Design Activity"
    mime = "model/vnd.collada+xml"
    extensions = [
        "dae",
    ]
    kind = Model

    format = Commodore64Cartridge
    signatures = [
        signature = b"C64 CARTRIDGE"
    ]
    name = "Commodore 64 Cartridge"
    short_name = "Commodore 64 Cartridge"
    mime = "application/x-commodore-64-cartridge"
    extensions = [
        "crt",
    ]
    kind = Rom

    format = Commodore64Program
    signatures = [
        signature = b"\x01\x08", b"\x9E" offset = 6
    ]
    name = "Commodore 64 Program"
    short_name = "Commodore 64 Program"
    mime = "application/x-commodore-64-program"
    extensions = [
        "prg",
    ]
    kind = Executable

    format = CommonObjectFileFormat
    signatures = [
        signature = b"\x00\x02"
        signature = b"\x4C\x01"
        signature = b"\x64\x86"
    ]
    name = "Common Object File Format"
    short_name = "Common Object File Format"
    mime = "application/x-coff"
    extensions = [
        "coff",
    ]
    kind = Executable

    format = CompoundFileBinary
    signatures = [
        signature = b"\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1"
    ]
    name = "Compound File Binary"
    short_name = "Compound File Binary"
    mime = "application/x-cfb"
    extensions = [
        "cfb",
    ]
    kind = Other

    format = CorelPresentations
    signatures = [
        signature = b"\xFFWPC", b"\x0F" offset = 9
        signature = b"\xFFWPC", b"\x10" offset = 9
    ]
    name = "Corel Presentations"
    short_name = "Corel Presentations"
    mime = "application/x-corelpresentations"
    extensions = [
        "shw",
    ]
    kind = Presentation

    format = CorelPresentations7
    signatures = []
    name = "Corel Presentations 7"
    short_name = "Corel Presentations 7"
    mime = "application/x-corelpresentations"
    extensions = [
        "shw",
    ]
    kind = Presentation

    format = Cpio
    signatures = [
        signature = b"0707"
        signature = b"\x71\xC7"
        signature = b"\xC7\x71"
    ]
    name = "cpio"
    short_name = "cpio"
    mime = "application/x-cpio"
    extensions = [
        "cpio",
    ]
    kind = Archive

    format = CreativeVoice
    signatures = [
        signature = b"Creative Voice File\x1A\x1A\0"
    ]
    name = "Creative Voice"
    short_name = "Creative Voice"
    mime = "audio/x-voc"
    extensions = [
        "voc",
    ]
    kind = Audio

    format = DalvikExecutable
    signatures = [
        signature = b"dex\n", b"\0" offset = 7
    ]
    name = "Dalvik Executable"
    short_name = "Dalvik Executable"
    mime = "application/vnd.android.dex"
    extensions = [
        "dex",
    ]
    kind = Executable

    format = DebianPackage
    signatures = [
        signature = b"!<arch>\ndebian-binary"
    ]
    name = "Debian Package"
    short_name = "Debian Package"
    mime = "application/vnd.debian.binary-package"
    extensions = [
        "deb",
    ]
    kind = Package

    format = DerCertificate
    signatures = [
        signature = b"\x30\x82"
    ]
    name = "DER Certificate"
    short_name = "DER Certificate"
    mime = "application/x-x509-ca-cert"
    extensions = [
        "der",
    ]
    kind = Other

    format = DesignWebFormat
    signatures = [
        signature = b"(DWF V"
    ]
    name = "Design Web Format"
    short_name = "Design Web Format"
    mime = "model/vnd.dwf"
    extensions = [
        "dwf",
    ]
    kind = Model

    format = DesignWebFormatXps
    signatures = []
    name = "Design Web Format XPS"
    short_name = "Design Web Format XPS"
    mime = "model/vnd.dwfx+xps"
    extensions = [
        "dwfx",
    ]
    kind = Model

    format = DigitalImagingAndCommunicationsInMedicine
    signatures = [
        signature = b"DICM" offset = 128
    ]
    name = "Digital Imaging and Communications in Medicine"
    short_name = "Digital Imaging and Communications in Medicine"
    mime = "application/dicom"
    extensions = [
        "dcm",
    ]
    kind = Other

    format = DigitalPictureExchange
    signatures = [
        signature = b"SDPX"
        signature = b"XPDS"
    ]
    name = "Digital Picture Exchange"
    short_name = "Digital Picture Exchange"
    mime = "image/x-dpx"
    extensions = [
        "dpx",
    ]
    kind = Image

    format = Djvu
    signatures = [
        signature = b"AT&TFORM", b"DJVI" offset = 12
        signature = b"AT&TFORM", b"DJVM" offset = 12
        signature = b"AT&TFORM", b"DJVU" offset = 12
        signature = b"AT&TFORM", b"THUM" offset = 12
    ]
    name = "DjVu"
    short_name = "DjVu"
    mime = "image/vnd.djvu"
    extensions = [
        "djvu",
    ]
    kind = Document

    format = DrawingExchangeFormatAscii
    signatures = [
        signature = b"  0\r\nSECTION"
        signature = b"  0\nSECTION"
    ]
    name = "Drawing Exchange Format ASCII"
    short_name = "Drawing Exchange Format ASCII"
    mime = "application/x-dxf"
    extensions = [
        "dxf",
    ]
    kind = Model

    format = DrawingExchangeFormatBinary
    signatures = [
        signature = b"AutoCAD Binary DXF"
    ]
    name = "Drawing Exchange Format Binary"
    short_name = "Drawing Exchange Format Binary"
    mime = "application/x-dxf"
    extensions = [
        "dxf",
    ]
    kind = Model

    format = DrawIo
    signatures = [
        signature = b"\xEF\xBB\xBF<mxfile"
        signature = b"<mxfile"
    ]
    name = "draw.io"
    short_name = "draw.io"
    mime = "application/vnd.jgraph.mxfile"
    extensions = [
        "drawio",
    ]
    kind = Diagram

    format = DynamicLinkLibrary
    signatures = []
    name = "Dynamic Link Library"
    short_name = "Dynamic Link Library"
    mime = "application/vnd.microsoft.portable-executable"
    extensions = [
        "dll",
    ]
    kind = Executable

    format = EightBitSampledVoice
    signatures = [
        signature = b"FORM", b"8SVX" offset = 8
    ]
    name = "8-Bit Sampled Voice"
    short_name = "8-Bit Sampled Voice"
    mime = "audio/x-8svx"
    extensions = [
        "8svx",
    ]
    kind = Audio

    format = ElectronicPublication
    signatures = []
    name = "Electronic Publication"
    short_name = "Electronic Publication"
    mime = "application/epub+zip"
    extensions = [
        "epub",
    ]
    kind = Ebook

    format = EmbeddedOpentype
    signatures = [
        signature = b"\x00\x00\x01" offset = 8, b"LP" offset = 34
        signature = b"\x01\x00\x02" offset = 8, b"LP" offset = 34
        signature = b"\x02\x00\x02" offset = 8, b"LP" offset = 34
    ]
    name = "Embedded OpenType"
    short_name = "Embedded OpenType"
    mime = "application/vnd.ms-fontobject"
    extensions = [
        "eot",
    ]
    kind = Font

    format = Empty
    signatures = []
    name = "Empty"
    short_name = "Empty"
    mime = "application/x-empty"
    extensions = [
        "empty",
    ]
    kind = Other

    format = EncapsulatedPostscript
    signatures = [
        signature = b"%!PS-Adobe-", b" EPSF-" offset = 14
        signature = b"\xC5\xD0\xD3\xC6"
    ]
    name = "Encapsulated PostScript"
    short_name = "Encapsulated PostScript"
    mime = "image/x-eps"
    extensions = [
        "eps",
    ]
    kind = Image

    format = EnhancedMetafile
    signatures = [
        signature = b"\x01\x00\x00\x00", b" EMF" offset = 40
    ]
    name = "Enhanced Metafile"
    short_name = "Enhanced Metafile"
    mime = "image/emf"
    extensions = [
        "emf",
    ]
    kind = Image

    format = EnterpriseApplicationArchive
    signatures = []
    name = "Enterprise Application Archive"
    short_name = "Enterprise Application Archive"
    mime = "application/java-archive"
    extensions = [
        "ear",
    ]
    kind = Package

    format = ExecutableAndLinkableFormat
    signatures = [
        signature = b"\x7FELF"
    ]
    name = "Executable and Linkable Format"
    short_name = "Executable and Linkable Format"
    mime = "application/x-executable"
    extensions = [
        "elf",
    ]
    kind = Executable

    format = ExperimentalComputingFacility
    signatures = [
        signature = b"gimp xcf"
    ]
    name = "Experimental Computing Facility"
    short_name = "Experimental Computing Facility"
    mime = "image/x-xcf"
    extensions = [
        "xcf",
    ]
    kind = Image

    format = Extensible3d
    signatures = [
        signature = b"\xEF\xBB\xBF<!DOCTYPE X3D PUBLIC"
        signature = b"<!DOCTYPE X3D PUBLIC"
        signature = b"\xEF\xBB\xBF<X3D"
        signature = b"\xEF\xBB\xBF<x3d"
        signature = b"<X3D"
        signature = b"<x3d"
    ]
    name = "Extensible 3D"
    short_name = "Extensible 3D"
    mime = "model/x3d+xml"
    extensions = [
        "x3d",
    ]
    kind = Model

    format = ExtensibleArchive
    signatures = [
        signature = b"xar!"
    ]
    name = "Extensible Archive"
    short_name = "Extensible Archive"
    mime = "application/x-xar"
    extensions = [
        "xar",
    ]
    kind = Archive

    format = ExtensibleBinaryMetaLanguage
    signatures = [
        signature = b"\x1A\x45\xDF\xA3"
    ]
    name = "Extensible Binary Meta Language"
    short_name = "Extensible Binary Meta Language"
    mime = "application/x-ebml"
    extensions = [
        "ebml",
    ]
    kind = Other

    format = ExtensibleMarkupLanguage
    signatures = [
        signature = b"\xEF\xBB\xBF<?xml"
        signature = b"<?xml"
    ]
    name = "Extensible Markup Language"
    short_name = "Extensible Markup Language"
    mime = "text/xml"
    extensions = [
        "xml",
    ]
    kind = Other

    format = ExtensibleStylesheetLanguageTransformations
    signatures = [
        signature = b"\xEF\xBB\xBF<xsl"
        signature = b"<xsl"
    ]
    name = "Extensible Stylesheet Language Transformations"
    short_name = "Extensible Stylesheet Language Transformations"
    mime = "application/xslt+xml"
    extensions = [
        "xsl",
    ]
    kind = Other

    format = Farbfeld
    signatures = [
        signature = b"farbfeld"
    ]
    name = "farbfeld"
    short_name = "farbfeld"
    mime = "image/x-ff"
    extensions = [
        "ff",
    ]
    kind = Image

    format = Fasttracker2ExtendedModule
    signatures = [
        signature = b"Extended Module:"
    ]
    name = "FastTracker 2 Extended Module"
    short_name = "FastTracker 2 Extended Module"
    mime = "audio/x-xm"
    extensions = [
        "xm",
    ]
    kind = Audio

    format = Fictionbook
    signatures = [
        signature = b"\xEF\xBB\xBF<FictionBook"
        signature = b"<FictionBook"
    ]
    name = "FictionBook"
    short_name = "FictionBook"
    mime = "application/x-fb2+xml"
    extensions = [
        "fb2",
    ]
    kind = Ebook

    format = FictionbookZip
    signatures = []
    name = "FictionBook ZIP"
    short_name = "FictionBook ZIP"
    mime = "application/x-fbz"
    extensions = [
        "fbz",
    ]
    kind = Ebook

    format = FigletFont
    signatures = [
        signature = b"flf2a"
    ]
    name = "FIGlet Font"
    short_name = "FIGlet Font"
    mime = "application/x-figlet"
    extensions = [
        "flf",
    ]
    kind = Font

    format = FigmaDesign
    signatures = []
    name = "Figma Design"
    short_name = "Figma Design"
    mime = "image/x-figma"
    extensions = [
        "fig",
    ]
    kind = Image

    format = Filmbox
    signatures = [
        signature = b"Kaydara FBX Binary  \0"
    ]
    name = "Filmbox"
    short_name = "Filmbox"
    mime = "application/vnd.autodesk.fbx"
    extensions = [
        "fbx",
    ]
    kind = Model

    format = FlashCs5Project
    signatures = []
    name = "Flash CS5 Project"
    short_name = "Flash CS5 Project"
    mime = "application/vnd.adobe.fla"
    extensions = [
        "fla",
    ]
    kind = Other

    format = FlashMp4Audio
    signatures = [
        signature = b"ftypF4A" offset = 4
        signature = b"ftypf4a" offset = 4
    ]
    name = "Flash MP4 Audio"
    short_name = "Flash MP4 Audio"
    mime = "audio/mp4"
    extensions = [
        "f4a",
    ]
    kind = Audio

    format = FlashMp4Audiobook
    signatures = [
        signature = b"ftypF4B" offset = 4
        signature = b"ftypf4b" offset = 4
    ]
    name = "Flash MP4 Audiobook"
    short_name = "Flash MP4 Audiobook"
    mime = "audio/mp4"
    extensions = [
        "f4b",
    ]
    kind = Audio

    format = FlashMp4ProtectedVideo
    signatures = [
        signature = b"ftypF4P" offset = 4
        signature = b"ftypf4p" offset = 4
    ]
    name = "Flash MP4 Protected Video"
    short_name = "Flash MP4 Protected Video"
    mime = "video/mp4"
    extensions = [
        "f4p",
    ]
    kind = Video

    format = FlashMp4Video
    signatures = [
        signature = b"ftypF4V" offset = 4
        signature = b"ftypf4v" offset = 4
    ]
    name = "Flash MP4 Video"
    short_name = "Flash MP4 Video"
    mime = "video/mp4"
    extensions = [
        "f4v",
    ]
    kind = Video

    format = FlashProject
    signatures = []
    name = "Flash Project"
    short_name = "Flash Project"
    mime = "application/vnd.adobe.fla"
    extensions = [
        "fla",
    ]
    kind = Other

    format = FlashVideo
    signatures = [
        signature = b"FLV\x01"
    ]
    name = "Flash Video"
    short_name = "Flash Video"
    mime = "video/x-flv"
    extensions = [
        "flv",
    ]
    kind = Video

    format = FlexibleAndInteroperableDataTransfer
    signatures = [
        signature = b".FIT" offset = 8
    ]
    name = "Flexible and Interoperable Data Transfer"
    short_name = "Flexible and Interoperable Data Transfer"
    mime = "application/x-fit"
    extensions = [
        "fit",
    ]
    kind = Geospatial

    format = FlexibleImageTransportSystem
    signatures = [
        signature = b"SIMPLE  =                    T"
    ]
    name = "Flexible Image Transport System"
    short_name = "Flexible Image Transport System"
    mime = "application/fits"
    extensions = [
        "fits",
    ]
    kind = Other

    format = FreeLosslessAudioCodec
    signatures = [
        signature = b"fLaC"
    ]
    name = "Free Lossless Audio Codec"
    short_name = "Free Lossless Audio Codec"
    mime = "audio/flac"
    extensions = [
        "flac",
    ]
    kind = Audio

    format = FreeLosslessImageFormat
    signatures = [
        signature = b"FLIF"
    ]
    name = "Free Lossless Image Format"
    short_name = "Free Lossless Image Format"
    mime = "image/flif"
    extensions = [
        "flif",
    ]
    kind = Image

    format = FujifilmRaw
    signatures = [
        signature = b"FUJIFILMCCD-RAW"
    ]
    name = "Fujifilm Raw"
    short_name = "Fujifilm Raw"
    mime = "image/x-fuji-raf"
    extensions = [
        "raf",
    ]
    kind = Image

    format = Fusion360
    signatures = []
    name = "Fusion 360"
    short_name = "Fusion 360"
    mime = "model/x-f3d"
    extensions = [
        "f3d",
    ]
    kind = Model

    format = GameBoyAdvanceRom
    signatures = [
        signature = b"\x24\xFF\xAE\x51\x69\x9A\xA2\x21" offset = 4
    ]
    name = "Game Boy Advance ROM"
    short_name = "Game Boy Advance ROM"
    mime = "application/x-gba-rom"
    extensions = [
        "gba",
    ]
    kind = Rom

    format = GameBoyColorRom
    signatures = [
        signature = b"\xCE\xED\x66\x66\xCC\x0D\x00\x0B" offset = 260, b"\x80" offset = 323
        signature = b"\xCE\xED\x66\x66\xCC\x0D\x00\x0B" offset = 260, b"\xC0" offset = 323
    ]
    name = "Game Boy Color ROM"
    short_name = "Game Boy Color ROM"
    mime = "application/x-gameboy-color-rom"
    extensions = [
        "gbc",
    ]
    kind = Rom

    format = GameBoyRom
    signatures = [
        signature = b"\xCE\xED\x66\x66\xCC\x0D\x00\x0B" offset = 260
    ]
    name = "Game Boy ROM"
    short_name = "Game Boy ROM"
    mime = "application/x-gameboy-rom"
    extensions = [
        "gb",
    ]
    kind = Rom

    format = GameGearRom
    signatures = [
        signature = b"TMR SEGA" offset = 32752, b"\x50" offset = 32767
        signature = b"TMR SEGA" offset = 32752, b"\x51" offset = 32767
        signature = b"TMR SEGA" offset = 32752, b"\x5C" offset = 32767
        signature = b"TMR SEGA" offset = 32752, b"\x5E" offset = 32767
        signature = b"TMR SEGA" offset = 32752, b"\x5F" offset = 32767
        signature = b"TMR SEGA" offset = 32752, b"\x60" offset = 32767
        signature = b"TMR SEGA" offset = 32752, b"\x61" offset = 32767
        signature = b"TMR SEGA" offset = 32752, b"\x6C" offset = 32767
        signature = b"TMR SEGA" offset = 32752, b"\x6E" offset = 32767
        signature = b"TMR SEGA" offset = 32752, b"\x6F" offset = 32767
        signature = b"TMR SEGA" offset = 32752, b"\x70" offset = 32767
        signature = b"TMR SEGA" offset = 32752, b"\x71" offset = 32767
        signature = b"TMR SEGA" offset = 32752, b"\x7C" offset = 32767
        signature = b"TMR SEGA" offset = 32752, b"\x7E" offset = 32767
        signature = b"TMR SEGA" offset = 32752, b"\x7F" offset = 32767
    ]
    name = "Game Gear ROM"
    short_name = "Game Gear ROM"
    mime = "application/x-gamegear-rom"
    extensions = [
        "gg",
    ]
    kind = Rom

    format = GeographyMarkupLanguage
    signatures = [
        signature = b"\xEF\xBB\xBF<gml"
        signature = b"<gml"
    ]
    name = "Geography Markup Language"
    short_name = "Geography Markup Language"
    mime = "application/gml+xml"
    extensions = [
        "gml",
    ]
    kind = Geospatial

    format = GettextMachineObject
    signatures = [
        signature = b"\x95\x04\x12\xDE"
        signature = b"\xDE\x12\x04\x95"
    ]
    name = "gettext Machine Object"
    short_name = "gettext Machine Object"
    mime = "application/x-gettext-translation"
    extensions = [
        "mo",
    ]
    kind = Other

    format = GlTransmissionFormatBinary
    signatures = [
        signature = b"glTF"
    ]
    name = "GL Transmission Format Binary"
    short_name = "GL Transmission Format Binary"
    mime = "model/gltf-binary"
    extensions = [
        "glb",
    ]
    kind = Model

    format = Glyphs
    signatures = [
        signature = b"{\r\n.appVersion = \""
        signature = b"{\n.appVersion = \""
        signature = b"{\r.appVersion = \""
    ]
    name = "Glyphs"
    short_name = "Glyphs"
    mime = "font/x-glyphs"
    extensions = [
        "glyphs",
    ]
    kind = Font

    format = GoogleChromeExtension
    signatures = [
        signature = b"Cr24"
    ]
    name = "Google Chrome Extension"
    short_name = "Google Chrome Extension"
    mime = "application/x-google-chrome-extension"
    extensions = [
        "crx",
    ]
    kind = Package

    format = GoogleDraco
    signatures = [
        signature = b"DRACO"
    ]
    name = "Google Draco"
    short_name = "Google Draco"
    mime = "model/x-draco"
    extensions = [
        "drc",
    ]
    kind = Model

    format = GpsExchangeFormat
    signatures = [
        signature = b"\xEF\xBB\xBF<gpx"
        signature = b"<gpx"
    ]
    name = "GPS Exchange Format"
    short_name = "GPS Exchange Format"
    mime = "application/gpx+xml"
    extensions = [
        "gpx",
    ]
    kind = Geospatial

    format = GraphicsInterchangeFormat
    signatures = [
        signature = b"GIF87a"
        signature = b"GIF89a"
    ]
    name = "Graphics Interchange Format"
    short_name = "Graphics Interchange Format"
    mime = "image/gif"
    extensions = [
        "gif",
    ]
    kind = Image

    format = Gzip
    signatures = [
        signature = b"\x1F\x8B"
    ]
    name = "gzip"
    short_name = "gzip"
    mime = "application/gzip"
    extensions = [
        "gz",
    ]
    kind = Compressed

    format = HighEfficiencyImageCoding
    signatures = [
        signature = b"ftypheic" offset = 4
        signature = b"ftypheix" offset = 4
    ]
    name = "High Efficiency Image Coding"
    short_name = "High Efficiency Image Coding"
    mime = "image/heic"
    extensions = [
        "heic",
    ]
    kind = Image

    format = HighEfficiencyImageCodingSequence
    signatures = [
        signature = b"ftyphevc" offset = 4
        signature = b"ftyphevx" offset = 4
    ]
    name = "High Efficiency Image Coding Sequence"
    short_name = "High Efficiency Image Coding Sequence"
    mime = "image/heic-sequence"
    extensions = [
        "heics",
    ]
    kind = Image

    format = HighEfficiencyImageFileFormat
    signatures = [
        signature = b"ftypavic" offset = 4
        signature = b"ftypheim" offset = 4
        signature = b"ftypheis" offset = 4
        signature = b"ftypmif1" offset = 4
    ]
    name = "High Efficiency Image File Format"
    short_name = "High Efficiency Image File Format"
    mime = "image/heif"
    extensions = [
        "heif",
    ]
    kind = Image

    format = HighEfficiencyImageFileFormatSequence
    signatures = [
        signature = b"ftypavcs" offset = 4
        signature = b"ftyphevm" offset = 4
        signature = b"ftyphevs" offset = 4
        signature = b"ftypmsf1" offset = 4
    ]
    name = "High Efficiency Image File Format Sequence"
    short_name = "High Efficiency Image File Format Sequence"
    mime = "image/heif-sequence"
    extensions = [
        "heifs",
    ]
    kind = Image

    format = HypertextMarkupLanguage
    signatures = [
        signature = b"\xEF\xBB\xBF<!DOCTYPE HTML"
        signature = b"\xEF\xBB\xBF<!DOCTYPE html"
        signature = b"\xEF\xBB\xBF<!doctype HTML"
        signature = b"\xEF\xBB\xBF<!doctype html"
        signature = b"\xEF\xBB\xBF<html"
        signature = b"\xEF\xBB\xBF<HTML"
        signature = b"<!DOCTYPE HTML"
        signature = b"<!DOCTYPE html"
        signature = b"<!doctype HTML"
        signature = b"<!doctype html"
        signature = b"<html"
        signature = b"<HTML"
    ]
    name = "HyperText Markup Language"
    short_name = "HyperText Markup Language"
    mime = "text/html"
    extensions = [
        "html",
    ]
    kind = Other

    format = Icalendar
    signatures = [
        signature = b"BEGIN:VCALENDAR"
    ]
    name = "iCalendar"
    short_name = "iCalendar"
    mime = "text/calendar"
    extensions = [
        "ics",
    ]
    kind = Other

    format = IccProfile
    signatures = [
        signature = b"acsp" offset = 36
    ]
    name = "ICC Profile"
    short_name = "ICC Profile"
    mime = "application/vnd.iccprofile"
    extensions = [
        "icc",
    ]
    kind = Other

    format = Id3v2
    signatures = [
        signature = b"ID3\x02"
        signature = b"ID3\x03"
        signature = b"ID3\x04"
    ]
    name = "ID3v2"
    short_name = "ID3v2"
    mime = "application/x-id3v2"
    extensions = [
        "id3",
    ]
    kind = Metadata

    format = ImpulseTrackerModule
    signatures = [
        signature = b"IMPM"
    ]
    name = "Impulse Tracker Module"
    short_name = "Impulse Tracker Module"
    mime = "audio/x-it"
    extensions = [
        "it",
    ]
    kind = Audio

    format = IndesignMarkupLanguage
    signatures = []
    name = "InDesign Markup Language"
    short_name = "InDesign Markup Language"
    mime = "application/vnd.adobe.indesign-idml-package"
    extensions = [
        "idml",
    ]
    kind = Document

    format = InitialGraphicsExchangeSpecification
    signatures = [
        signature = b"S      1\r\n" offset = 72
        signature = b"S0000001\r\n" offset = 72
        signature = b"S      1\n" offset = 72
        signature = b"S      1\r" offset = 72
        signature = b"S0000001\n" offset = 72
        signature = b"S0000001\r" offset = 72
    ]
    name = "Initial Graphics Exchange Specification"
    short_name = "Initial Graphics Exchange Specification"
    mime = "model/iges"
    extensions = [
        "iges",
    ]
    kind = Model

    format = InterQuakeExport
    signatures = [
        signature = b"# Inter-Quake Export"
    ]
    name = "Inter-Quake Export"
    short_name = "Inter-Quake Export"
    mime = "model/x-iqe"
    extensions = [
        "iqe",
    ]
    kind = Model

    format = InterQuakeModel
    signatures = [
        signature = b"INTERQUAKEMODEL\0"
    ]
    name = "Inter-Quake Model"
    short_name = "Inter-Quake Model"
    mime = "model/x-iqm"
    extensions = [
        "iqm",
    ]
    kind = Model

    format = IosAppStorePackage
    signatures = []
    name = "iOS App Store Package"
    short_name = "iOS App Store Package"
    mime = "application/x-ios-app"
    extensions = [
        "ipa",
    ]
    kind = Package

    format = Iso9660
    signatures = [
        signature = b"CD001" offset = 32769
        signature = b"CD001" offset = 34817
        signature = b"CD001" offset = 36865
    ]
    name = "ISO 9660"
    short_name = "ISO 9660"
    mime = "application/x-iso9660-image"
    extensions = [
        "iso",
    ]
    kind = Disk

    format = JavaArchive
    signatures = []
    name = "Java Archive"
    short_name = "Java Archive"
    mime = "application/java-archive"
    extensions = [
        "jar",
    ]
    kind = Package

    format = JavaClass
    signatures = [
        signature = b"\xCA\xFE\xBA\xBE"
    ]
    name = "Java Class"
    short_name = "Java Class"
    mime = "application/java-vm"
    extensions = [
        "class",
    ]
    kind = Executable

    format = JavaKeystore
    signatures = [
        signature = b"\xFE\xED\xFE\xED"
    ]
    name = "Java KeyStore"
    short_name = "Java KeyStore"
    mime = "application/x-java-keystore"
    extensions = [
        "jks",
    ]
    kind = Other

    format = JointPhotographicExpertsGroup
    signatures = [
        signature = b"\xFF\xD8\xFF"
    ]
    name = "Joint Photographic Experts Group"
    short_name = "Joint Photographic Experts Group"
    mime = "image/jpeg"
    extensions = [
        "jpg",
    ]
    kind = Image

    format = Jpeg2000Codestream
    signatures = [
        signature = b"\xFF\x4F\xFF\x51"
    ]
    name = "JPEG 2000 Codestream"
    short_name = "JPEG 2000 Codestream"
    mime = "image/x-jp2-codestream"
    extensions = [
        "j2c",
    ]
    kind = Image

    format = Jpeg2000Part1
    signatures = [
        signature = b"ftypJP2 " offset = 16
        signature = b"ftypjp2 " offset = 16
    ]
    name = "JPEG 2000 Part 1"
    short_name = "JPEG 2000 Part 1"
    mime = "image/jp2"
    extensions = [
        "jp2",
    ]
    kind = Image

    format = Jpeg2000Part2
    signatures = [
        signature = b"ftypjpx " offset = 16
    ]
    name = "JPEG 2000 Part 2"
    short_name = "JPEG 2000 Part 2"
    mime = "image/jpx"
    extensions = [
        "jpx",
    ]
    kind = Image

    format = Jpeg2000Part3
    signatures = [
        signature = b"ftypmjp2" offset = 16
    ]
    name = "JPEG 2000 Part 3"
    short_name = "JPEG 2000 Part 3"
    mime = "video/mj2"
    extensions = [
        "mj2",
    ]
    kind = Video

    format = Jpeg2000Part6
    signatures = [
        signature = b"ftypjpm " offset = 16
    ]
    name = "JPEG 2000 Part 6"
    short_name = "JPEG 2000 Part 6"
    mime = "image/jpm"
    extensions = [
        "jpm",
    ]
    kind = Image

    format = JpegExtendedRange
    signatures = [
        signature = b"\x49\x49\xBC"
    ]
    name = "JPEG Extended Range"
    short_name = "JPEG Extended Range"
    mime = "image/jxr"
    extensions = [
        "jxr",
    ]
    kind = Image

    format = JpegLs
    signatures = [
        signature = b"\xFF\xD8\xFF\xF7"
    ]
    name = "JPEG-LS"
    short_name = "JPEG-LS"
    mime = "image/jls"
    extensions = [
        "jls",
    ]
    kind = Image

    format = JpegNetworkGraphics
    signatures = [
        signature = b"\x8BJNG\r\n\x1A\n"
    ]
    name = "JPEG Network Graphics"
    short_name = "JPEG Network Graphics"
    mime = "image/x-jng"
    extensions = [
        "jng",
    ]
    kind = Image

    format = JpegXl
    signatures = [
        signature = b"\0\0\0\x0CJXL \r\n\x87\n"
        signature = b"\xFF\n"
    ]
    name = "JPEG XL"
    short_name = "JPEG XL"
    mime = "image/jxl"
    extensions = [
        "jxl",
    ]
    kind = Image

    format = JsonFeed
    signatures = [
        signature = b"{\r\n    \"version\": \"https://jsonfeed.org/version/"
        signature = b"{\n    \"version\": \"https://jsonfeed.org/version/"
        signature = b"{\r    \"version\": \"https://jsonfeed.org/version/"
        signature = b"{\r\n  \"version\": \"https://jsonfeed.org/version/"
        signature = b"{\r\n\t\"version\": \"https://jsonfeed.org/version/"
        signature = b"{\n  \"version\": \"https://jsonfeed.org/version/"
        signature = b"{\n\t\"version\": \"https://jsonfeed.org/version/"
        signature = b"{\r  \"version\": \"https://jsonfeed.org/version/"
        signature = b"{\r\t\"version\": \"https://jsonfeed.org/version/"
        signature = b"{\"version\":\"https://jsonfeed.org/version/"
    ]
    name = "JSON Feed"
    short_name = "JSON Feed"
    mime = "application/feed+json"
    extensions = [
        "json",
    ]
    kind = Other

    format = KeePassDataBase
    signatures = [
        signature = b"\x03\xD9\xA2\x9A\x67\xFB\x4B\xB5"
    ]
    name = "KeePass DataBase"
    short_name = "KDBX"
    mime = "application/x-keepass2"
    extensions = [
        "kdbx"
    ]
    kind = Database

    format = KeyholeMarkupLanguage
    signatures = [
        signature = b"\xEF\xBB\xBF<kml"
        signature = b"<kml"
    ]
    name = "Keyhole Markup Language"
    short_name = "Keyhole Markup Language"
    mime = "application/vnd.google-earth.kml+xml"
    extensions = [
        "kml",
    ]
    kind = Geospatial

    format = KeyholeMarkupLanguageZip
    signatures = []
    name = "Keyhole Markup Language ZIP"
    short_name = "Keyhole Markup Language ZIP"
    mime = "application/vnd.google-earth.kmz"
    extensions = [
        "kmz",
    ]
    kind = Geospatial

    format = KhronosTexture
    signatures = [
        signature = b"\xABKTX 11\xBB\r\n\x1A\n"
    ]
    name = "Khronos Texture"
    short_name = "Khronos Texture"
    mime = "image/ktx"
    extensions = [
        "ktx",
    ]
    kind = Image

    format = KhronosTexture2
    signatures = [
        signature = b"\xABKTX 20\xBB\r\n\x1A\n"
    ]
    name = "Khronos Texture 2"
    short_name = "Khronos Texture 2"
    mime = "image/ktx2"
    extensions = [
        "ktx2",
    ]
    kind = Image

    format = Larc
    signatures = [
        signature = b"-lz2-" offset = 2
        signature = b"-lz3-" offset = 2
        signature = b"-lz4-" offset = 2
        signature = b"-lz5-" offset = 2
        signature = b"-lz7-" offset = 2
        signature = b"-lz8-" offset = 2
        signature = b"-lzs-" offset = 2
    ]
    name = "LArc"
    short_name = "LArc"
    mime = "application/x-lzh-compressed"
    extensions = [
        "lzs",
    ]
    kind = Archive

    format = Latex
    signatures = [
        signature = b"\\documentclass"
        signature = b"\\documentstyle"
    ]
    name = "LaTeX"
    short_name = "LaTeX"
    mime = "text/x-tex"
    extensions = [
        "tex",
    ]
    kind = Document

    format = LempelZivFiniteStateEntropy
    signatures = [
        signature = b"bvx-"
        signature = b"bvx1"
        signature = b"bvx2"
        signature = b"bvxn"
    ]
    name = "Lempel-Ziv Finite State Entropy"
    short_name = "Lempel-Ziv Finite State Entropy"
    mime = "application/x-lzfse"
    extensions = [
        "lzfse",
    ]
    kind = Compressed

    format = LempelZivMarkovChainAlgorithm
    signatures = [
        signature = b"\x5D\x00\x00\x80\x00"
    ]
    name = "Lempel-Ziv-Markov chain algorithm"
    short_name = "Lempel-Ziv-Markov chain algorithm"
    mime = "application/x-lzma"
    extensions = [
        "lzma",
    ]
    kind = Compressed

    format = Lha
    signatures = [
        signature = b"-lh0-" offset = 2
        signature = b"-lh1-" offset = 2
        signature = b"-lh2-" offset = 2
        signature = b"-lh3-" offset = 2
        signature = b"-lh4-" offset = 2
        signature = b"-lh5-" offset = 2
        signature = b"-lh6-" offset = 2
        signature = b"-lh7-" offset = 2
        signature = b"-lh8-" offset = 2
        signature = b"-lhd-" offset = 2
    ]
    name = "LHA"
    short_name = "LHA"
    mime = "application/x-lzh-compressed"
    extensions = [
        "lzh",
    ]
    kind = Archive

    format = LinearExecutable
    signatures = []
    name = "Linear Executable"
    short_name = "Linear Executable"
    mime = "application/x-dosexec"
    extensions = [
        "exe",
    ]
    kind = Executable

    format = LlvmBitcode
    signatures = [
        signature = b"BC\xC0\xDE"
    ]
    name = "LLVM Bitcode"
    short_name = "LLVM Bitcode"
    mime = "application/x-llvm"
    extensions = [
        "bc",
    ]
    kind = Executable

    format = LongRangeZip
    signatures = [
        signature = b"LRZI"
    ]
    name = "Long Range ZIP"
    short_name = "Long Range ZIP"
    mime = "application/x-lrzip"
    extensions = [
        "lrz",
    ]
    kind = Compressed

    format = LuaBytecode
    signatures = [
        signature = b"\x1BLua"
    ]
    name = "Lua Bytecode"
    short_name = "Lua Bytecode"
    mime = "application/x-lua-bytecode"
    extensions = [
        "luac",
    ]
    kind = Executable

    format = LuaScript
    signatures = [
        signature = b"#!/usr/local/bin/lua"
        signature = b"#!/usr/bin/env lua"
        signature = b"#!/usr/bin/lua"
    ]
    name = "Lua Script"
    short_name = "Lua Script"
    mime = "text/x-lua"
    extensions = [
        "lua",
    ]
    kind = Other

    format = Lz4
    signatures = [
        signature = b"\x04\x22\x4D\x18"
    ]
    name = "LZ4"
    short_name = "LZ4"
    mime = "application/x-lz4"
    extensions = [
        "lz4",
    ]
    kind = Compressed

    format = Lzip
    signatures = [
        signature = b"LZIP"
    ]
    name = "lzip"
    short_name = "lzip"
    mime = "application/x-lzip"
    extensions = [
        "lz",
    ]
    kind = Compressed

    format = Lzop
    signatures = [
        signature = b"\x89LZO\0\r\n\x1A\n"
    ]
    name = "lzop"
    short_name = "lzop"
    mime = "application/x-lzop"
    extensions = [
        "lzo",
    ]
    kind = Compressed

    format = MachO
    signatures = [
        signature = b"\xCA\xFE\xBA\xBE", b"\x01" offset = 7
        signature = b"\xCA\xFE\xBA\xBE", b"\x02" offset = 7
        signature = b"\xCA\xFE\xBA\xBE", b"\x03" offset = 7
        signature = b"\xCA\xFE\xBA\xBE", b"\x04" offset = 7
        signature = b"\xCA\xFE\xBA\xBE", b"\x05" offset = 7
        signature = b"\xCA\xFE\xBA\xBE", b"\x06" offset = 7
        signature = b"\xCA\xFE\xBA\xBE", b"\x07" offset = 7
        signature = b"\xCA\xFE\xBA\xBE", b"\x08" offset = 7
        signature = b"\xCA\xFE\xBA\xBE", b"\x09" offset = 7
        signature = b"\xCA\xFE\xBA\xBE", b"\x0A" offset = 7
        signature = b"\xCA\xFE\xBA\xBE", b"\x0B" offset = 7
        signature = b"\xCA\xFE\xBA\xBE", b"\x0C" offset = 7
        signature = b"\xCA\xFE\xBA\xBE", b"\x0D" offset = 7
        signature = b"\xCA\xFE\xBA\xBE", b"\x0E" offset = 7
        signature = b"\xCA\xFE\xBA\xBE", b"\x0F" offset = 7
        signature = b"\xCA\xFE\xBA\xBE", b"\x10" offset = 7
        signature = b"\xCA\xFE\xBA\xBE", b"\x11" offset = 7
        signature = b"\xCA\xFE\xBA\xBE", b"\x12" offset = 7
        signature = b"\xCA\xFE\xBA\xBE", b"\xFF" offset = 7
        signature = b"\xCE\xFA\xED\xFE"
        signature = b"\xCF\xFA\xED\xFE"
        signature = b"\xFE\xED\xFA\xCE"
        signature = b"\xFE\xED\xFA\xCF"
    ]
    name = "Mach-O"
    short_name = "Mach-O"
    mime = "application/x-mach-binary"
    extensions = [
        "mach",
    ]
    kind = Executable

    format = MacosAlias
    signatures = [
        signature = b"book\0\0\0\0mark\0\0\0\0"
    ]
    name = "macOS Alias"
    short_name = "macOS Alias"
    mime = "application/x-apple-alias"
    extensions = [
        "alias",
    ]
    kind = Metadata

    format = Magicavoxel
    signatures = [
        signature = b"VOX "
    ]
    name = "MagicaVoxel"
    short_name = "MagicaVoxel"
    mime = "model/x-vox"
    extensions = [
        "vox",
    ]
    kind = Model

    format = MagickImageFileFormat
    signatures = [
        signature = b"id=ImageMagick"
    ]
    name = "Magick Image File Format"
    short_name = "Magick Image File Format"
    mime = "image/x-miff"
    extensions = [
        "miff",
    ]
    kind = Image

    format = MaterialExchangeFormat
    signatures = [
        signature = b"\x06\x0E\x2B\x34\x02\x05\x01\x01\x0D\x01\x02\x01\x01\x02"
    ]
    name = "Material Exchange Format"
    short_name = "Material Exchange Format"
    mime = "application/mxf"
    extensions = [
        "mxf",
    ]
    kind = Video

    format = MathematicalMarkupLanguage
    signatures = [
        signature = b"\xEF\xBB\xBF<!DOCTYPE math PUBLIC"
        signature = b"<!DOCTYPE math PUBLIC"
        signature = b"\xEF\xBB\xBF<math"
        signature = b"<math"
    ]
    name = "Mathematical Markup Language"
    short_name = "Mathematical Markup Language"
    mime = "application/mathml+xml"
    extensions = [
        "mathml",
    ]
    kind = Formula

    format = Matroska3dVideo
    signatures = []
    name = "Matroska 3D Video"
    short_name = "Matroska 3D Video"
    mime = "video/matroska"
    extensions = [
        "mk3d",
    ]
    kind = Video

    format = MatroskaAudio
    signatures = []
    name = "Matroska Audio"
    short_name = "Matroska Audio"
    mime = "audio/matroska"
    extensions = [
        "mka",
    ]
    kind = Audio

    format = MatroskaSubtitles
    signatures = []
    name = "Matroska Subtitles"
    short_name = "Matroska Subtitles"
    mime = "application/x-matroska"
    extensions = [
        "mks",
    ]
    kind = Subtitle

    format = MatroskaVideo
    signatures = []
    name = "Matroska Video"
    short_name = "Matroska Video"
    mime = "video/matroska"
    extensions = [
        "mkv",
    ]
    kind = Video

    format = MayaAscii
    signatures = [
        signature = b"//Maya ASCII"
    ]
    name = "Maya ASCII"
    short_name = "Maya ASCII"
    mime = "application/x-maya-ascii"
    extensions = [
        "ma",
    ]
    kind = Model

    format = MayaBinary
    signatures = [
        signature = b"FOR4", b"MAYA" offset = 8
        signature = b"FOR4", b"Maya" offset = 8
        signature = b"FOR8", b"MAYA" offset = 16
        signature = b"FOR8", b"Maya" offset = 16
    ]
    name = "Maya Binary"
    short_name = "Maya Binary"
    mime = "application/x-maya-binary"
    extensions = [
        "mb",
    ]
    kind = Model

    format = MegaDriveRom
    signatures = [
        signature = b"SEGA MEGA DRIVE" offset = 256
        signature = b"SEGA GENESIS" offset = 256
    ]
    name = "Mega Drive ROM"
    short_name = "Mega Drive ROM"
    mime = "application/x-genesis-rom"
    extensions = [
        "md",
    ]
    kind = Rom

    format = MetaInformationEncapsulation
    signatures = [
        signature = b"\x7E\x10\x04", b"0MIE" offset = 4
        signature = b"\x7E\x18\x04", b"0MIE" offset = 4
    ]
    name = "Meta Information Encapsulation"
    short_name = "Meta Information Encapsulation"
    mime = "application/x-mie"
    extensions = [
        "mie",
    ]
    kind = Metadata

    format = MicrosoftAccess2007Database
    signatures = [
        signature = b"Standard ACE DB" offset = 4
    ]
    name = "Microsoft Access 2007 Database"
    short_name = "Microsoft Access 2007 Database"
    mime = "application/x-msaccess"
    extensions = [
        "accdb",
    ]
    kind = Database

    format = MicrosoftAccessDatabase
    signatures = [
        signature = b"Standard Jet DB" offset = 4
    ]
    name = "Microsoft Access Database"
    short_name = "Microsoft Access Database"
    mime = "application/x-msaccess"
    extensions = [
        "mdb",
    ]
    kind = Database

    format = MicrosoftCompiledHtmlHelp
    signatures = [
        signature = b"ITSF"
    ]
    name = "Microsoft Compiled HTML Help"
    short_name = "Microsoft Compiled HTML Help"
    mime = "application/vnd.ms-htmlhelp"
    extensions = [
        "chm",
    ]
    kind = Other

    format = MicrosoftDigitalVideoRecording
    signatures = []
    name = "Microsoft Digital Video Recording"
    short_name = "Microsoft Digital Video Recording"
    mime = "video/x-ms-asf"
    extensions = [
        "dvr-ms",
    ]
    kind = Video

    format = MicrosoftDirectdrawSurface
    signatures = [
        signature = b"DDS "
    ]
    name = "Microsoft DirectDraw Surface"
    short_name = "Microsoft DirectDraw Surface"
    mime = "image/vnd.ms-dds"
    extensions = [
        "dds",
    ]
    kind = Image

    format = MicrosoftExcelSpreadsheet
    signatures = []
    name = "Microsoft Excel Spreadsheet"
    short_name = "Microsoft Excel Spreadsheet"
    mime = "application/vnd.ms-excel"
    extensions = [
        "xls",
    ]
    kind = Spreadsheet

    format = MicrosoftPowerpointPresentation
    signatures = []
    name = "Microsoft PowerPoint Presentation"
    short_name = "Microsoft PowerPoint Presentation"
    mime = "application/vnd.ms-powerpoint"
    extensions = [
        "ppt",
    ]
    kind = Presentation

    format = MicrosoftProjectPlan
    signatures = []
    name = "Microsoft Project Plan"
    short_name = "Microsoft Project Plan"
    mime = "application/vnd.ms-project"
    extensions = [
        "mpp",
    ]
    kind = Other

    format = MicrosoftPublisherDocument
    signatures = []
    name = "Microsoft Publisher Document"
    short_name = "Microsoft Publisher Document"
    mime = "application/vnd.ms-publisher"
    extensions = [
        "pub",
    ]
    kind = Document

    format = MicrosoftReader
    signatures = [
        signature = b"ITOLITLS\x01\0\0\0(\0\0\0", b"\xC1\x07\x90\x0A\x76\x40\xD3\x11\x87\x89\x00\x00\xF8\x10\x57\x54" offset = 24
    ]
    name = "Microsoft Reader"
    short_name = "Microsoft Reader"
    mime = "application/x-ms-reader"
    extensions = [
        "lit",
    ]
    kind = Ebook

    format = MicrosoftSoftwareInstaller
    signatures = []
    name = "Microsoft Software Installer"
    short_name = "Microsoft Software Installer"
    mime = "application/x-msi"
    extensions = [
        "msi",
    ]
    kind = Package

    format = MicrosoftVirtualHardDisk
    signatures = [
        signature = b"connectix"
    ]
    name = "Microsoft Virtual Hard Disk"
    short_name = "Microsoft Virtual Hard Disk"
    mime = "application/x-vhd"
    extensions = [
        "vhd",
    ]
    kind = Disk

    format = MicrosoftVirtualHardDisk2
    signatures = [
        signature = b"vhdxfile"
    ]
    name = "Microsoft Virtual Hard Disk 2"
    short_name = "Microsoft Virtual Hard Disk 2"
    mime = "application/x-vhdx"
    extensions = [
        "vhdx",
    ]
    kind = Disk

    format = MicrosoftVisioDrawing
    signatures = []
    name = "Microsoft Visio Drawing"
    short_name = "Microsoft Visio Drawing"
    mime = "application/vnd.visio"
    extensions = [
        "vsd",
    ]
    kind = Diagram

    format = MicrosoftVisualStudioExtension
    signatures = []
    name = "Microsoft Visual Studio Extension"
    short_name = "Microsoft Visual Studio Extension"
    mime = "application/vsix"
    extensions = [
        "vsix",
    ]
    kind = Package

    format = MicrosoftVisualStudioSolution
    signatures = [
        signature = b"\xEF\xBB\xBF\r\nMicrosoft Visual Studio Solution File, Format Version "
        signature = b"\xEF\xBB\xBF\nMicrosoft Visual Studio Solution File, Format Version "
        signature = b"Microsoft Visual Studio Solution File, Format Version "
    ]
    name = "Microsoft Visual Studio Solution"
    short_name = "Microsoft Visual Studio Solution"
    mime = "application/vnd.ms-developer"
    extensions = [
        "sln",
    ]
    kind = Other

    format = MicrosoftWordDocument
    signatures = []
    name = "Microsoft Word Document"
    short_name = "Microsoft Word Document"
    mime = "application/msword"
    extensions = [
        "doc",
    ]
    kind = Document

    format = MicrosoftWorks6Spreadsheet
    signatures = []
    name = "Microsoft Works 6 Spreadsheet"
    short_name = "Microsoft Works 6 Spreadsheet"
    mime = "application/vnd.ms-works"
    extensions = [
        "xlr",
    ]
    kind = Spreadsheet

    format = MicrosoftWorksDatabase
    signatures = [
        signature = b"\x20\x54\x02\x00\x00\x00\x05\x54\x02\x00"
    ]
    name = "Microsoft Works Database"
    short_name = "Microsoft Works Database"
    mime = "application/vnd.ms-works-db"
    extensions = [
        "wdb",
    ]
    kind = Database

    format = MicrosoftWorksSpreadsheet
    signatures = [
        signature = b"\x00\x00\x02\x00\x04\x04\x05\x54\x02\x00"
        signature = b"\xFF\x00\x02\x00\x04\x04\x05\x54\x02\x00"
    ]
    name = "Microsoft Works Spreadsheet"
    short_name = "Microsoft Works Spreadsheet"
    mime = "application/vnd.ms-works"
    extensions = [
        "wks",
    ]
    kind = Spreadsheet

    format = MicrosoftWorksWordProcessor
    signatures = [
        signature = b"\x01\xFE", b"\x01\x00" offset = 112
    ]
    name = "Microsoft Works Word Processor"
    short_name = "Microsoft Works Word Processor"
    mime = "application/vnd.ms-works"
    extensions = [
        "wps",
    ]
    kind = Document

    format = MicrosoftWrite
    signatures = [
        signature = b"\x31\xBE\x00\x00\x00\xAB"
        signature = b"\x32\xBE\x00\x00\x00\xAB"
    ]
    name = "Microsoft Write"
    short_name = "Microsoft Write"
    mime = "application/x-mswrite"
    extensions = [
        "wri",
    ]
    kind = Document

    format = Mobipocket
    signatures = [
        signature = b"BOOKMOBI" offset = 60
    ]
    name = "Mobipocket"
    short_name = "Mobipocket"
    mime = "application/x-mobipocket-ebook"
    extensions = [
        "mobi",
    ]
    kind = Ebook

    format = Model3dAscii
    signatures = [
        signature = b"3dmodel"
    ]
    name = "Model 3D ASCII"
    short_name = "Model 3D ASCII"
    mime = "text/x-3d-model"
    extensions = [
        "a3d",
    ]
    kind = Model

    format = Model3dBinary
    signatures = [
        signature = b"3DMO"
    ]
    name = "Model 3D Binary"
    short_name = "Model 3D Binary"
    mime = "model/x-3d-model"
    extensions = [
        "m3d",
    ]
    kind = Model

    format = MonkeysAudio
    signatures = [
        signature = b"MAC "
    ]
    name = "Monkey's Audio"
    short_name = "Monkey's Audio"
    mime = "audio/x-ape"
    extensions = [
        "ape",
    ]
    kind = Audio

    format = MozillaArchive
    signatures = [
        signature = b"MAR1"
    ]
    name = "Mozilla Archive"
    short_name = "Mozilla Archive"
    mime = "application/x-mozilla-archive"
    extensions = [
        "mar",
    ]
    kind = Archive

    format = Mp3Url
    signatures = [
        signature = b"#EXTM3U"
    ]
    name = "MP3 URL"
    short_name = "MP3 URL"
    mime = "audio/x-mpegurl"
    extensions = [
        "m3u",
    ]
    kind = Playlist

    format = Mpeg12AudioLayer2
    signatures = [
        signature = b"\xFF\xF4"
        signature = b"\xFF\xF5"
        signature = b"\xFF\xFC"
        signature = b"\xFF\xFD"
    ]
    name = "MPEG-1/2 Audio Layer 2"
    short_name = "MPEG-1/2 Audio Layer 2"
    mime = "audio/mpeg"
    extensions = [
        "mp2",
    ]
    kind = Audio

    format = Mpeg12AudioLayer3
    signatures = [
        signature = b"\xFF\xE2"
        signature = b"\xFF\xE3"
        signature = b"\xFF\xF2"
        signature = b"\xFF\xF3"
        signature = b"\xFF\xFA"
        signature = b"\xFF\xFB"
    ]
    name = "MPEG-1/2 Audio Layer 3"
    short_name = "MPEG-1/2 Audio Layer 3"
    mime = "audio/mpeg"
    extensions = [
        "mp3",
    ]
    kind = Audio

    format = Mpeg12Video
    signatures = [
        signature = b"\x00\x00\x01\xB3"
        signature = b"\x00\x00\x01\xBA"
    ]
    name = "MPEG-1/2 Video"
    short_name = "MPEG-1/2 Video"
    mime = "video/mpeg"
    extensions = [
        "mpg",
    ]
    kind = Video

    format = Mpeg2TransportStream
    signatures = [
        signature = b"\x47", b"\x47" offset = 188
    ]
    name = "MPEG-2 Transport Stream"
    short_name = "MPEG-2 Transport Stream"
    mime = "video/mp2t"
    extensions = [
        "ts",
    ]
    kind = Video

    format = Mpeg4Part14
    signatures = [
        signature = b"ftypARRI" offset = 4
        signature = b"ftypFACE" offset = 4
        signature = b"ftypMSNV" offset = 4
        signature = b"ftypNDAS" offset = 4
        signature = b"ftypNDSC" offset = 4
        signature = b"ftypNDSH" offset = 4
        signature = b"ftypNDSM" offset = 4
        signature = b"ftypNDSP" offset = 4
        signature = b"ftypNDSS" offset = 4
        signature = b"ftypNDXC" offset = 4
        signature = b"ftypNDXH" offset = 4
        signature = b"ftypNDXM" offset = 4
        signature = b"ftypNDXP" offset = 4
        signature = b"ftypNDXS" offset = 4
        signature = b"ftypXAVC" offset = 4
        signature = b"ftypavc1" offset = 4
        signature = b"ftypdash" offset = 4
        signature = b"ftypiso2" offset = 4
        signature = b"ftypiso3" offset = 4
        signature = b"ftypiso4" offset = 4
        signature = b"ftypiso5" offset = 4
        signature = b"ftypiso6" offset = 4
        signature = b"ftypisom" offset = 4
        signature = b"ftypmmp4" offset = 4
        signature = b"ftypmobi" offset = 4
        signature = b"ftypmp21" offset = 4
        signature = b"ftypmp41" offset = 4
        signature = b"ftypmp42" offset = 4
        signature = b"ftypmp4v" offset = 4
        signature = b"ftypmp71" offset = 4
        signature = b"ftypnvr1" offset = 4
    ]
    name = "MPEG-4 Part 14"
    short_name = "MPEG-4 Part 14"
    mime = "application/mp4"
    extensions = [
        "mp4",
    ]
    kind = Other

    format = Mpeg4Part14Audio
    signatures = []
    name = "MPEG-4 Part 14 Audio"
    short_name = "MPEG-4 Part 14 Audio"
    mime = "audio/mp4"
    extensions = [
        "mp4",
    ]
    kind = Audio

    format = Mpeg4Part14Subtitles
    signatures = []
    name = "MPEG-4 Part 14 Subtitles"
    short_name = "MPEG-4 Part 14 Subtitles"
    mime = "application/mp4"
    extensions = [
        "mp4",
    ]
    kind = Subtitle

    format = Mpeg4Part14Video
    signatures = []
    name = "MPEG-4 Part 14 Video"
    short_name = "MPEG-4 Part 14 Video"
    mime = "video/mp4"
    extensions = [
        "mp4",
    ]
    kind = Video

    format = MpegDashMpd
    signatures = [
        signature = b"\xEF\xBB\xBF<MPD"
        signature = b"<MPD"
    ]
    name = "MPEG-DASH MPD"
    short_name = "MPEG-DASH MPD"
    mime = "application/dash+xml"
    extensions = [
        "mpd",
    ]
    kind = Playlist

    format = MsDosBatch
    signatures = [
        signature = b"@ECHO OFF"
        signature = b"@echo off"
    ]
    name = "MS-DOS Batch"
    short_name = "MS-DOS Batch"
    mime = "text/x-msdos-batch"
    extensions = [
        "bat",
    ]
    kind = Other

    format = MsDosExecutable
    signatures = [
        signature = b"MZ"
        signature = b"ZM"
    ]
    name = "MS-DOS Executable"
    short_name = "MS-DOS Executable"
    mime = "application/x-dosexec"
    extensions = [
        "exe",
    ]
    kind = Executable

    format = Mtv
    signatures = [
        signature = b"AMV"
    ]
    name = "MTV"
    short_name = "MTV"
    mime = "video/x-mtv"
    extensions = [
        "mtv",
    ]
    kind = Video

    format = MultiLayerArchive
    signatures = [
        signature = b"MLA\x01\x00\x00\x00"
    ]
    name = "Multi Layer Archive"
    short_name = "Multi Layer Archive"
    mime = "application/x-mla"
    extensions = [
        "mla",
    ]
    kind = Archive

    format = MultipleImageNetworkGraphics
    signatures = [
        signature = b"\x8AMNG\r\n\x1A\n"
    ]
    name = "Multiple-image Network Graphics"
    short_name = "Multiple-image Network Graphics"
    mime = "image/x-mng"
    extensions = [
        "mng",
    ]
    kind = Image

    format = Musepack
    signatures = [
        signature = b"MPCK"
        signature = b"MP+"
    ]
    name = "Musepack"
    short_name = "Musepack"
    mime = "audio/x-musepack"
    extensions = [
        "mpc",
    ]
    kind = Audio

    format = MusicalInstrumentDigitalInterface
    signatures = [
        signature = b"MThd"
    ]
    name = "Musical Instrument Digital Interface"
    short_name = "Musical Instrument Digital Interface"
    mime = "audio/midi"
    extensions = [
        "mid",
    ]
    kind = Audio

    format = Musicxml
    signatures = [
        signature = b"\xEF\xBB\xBF<score-partwise"
        signature = b"<score-partwise"
    ]
    name = "MusicXML"
    short_name = "MusicXML"
    mime = "application/vnd.recordare.musicxml+xml"
    extensions = [
        "musicxml",
    ]
    kind = Other

    format = MusicxmlZip
    signatures = []
    name = "MusicXML ZIP"
    short_name = "MusicXML ZIP"
    mime = "application/vnd.recordare.musicxml"
    extensions = [
        "mxl",
    ]
    kind = Other

    format = NeoGeoPocketColorRom
    signatures = [
        signature = b" LICENSED BY SNK CORPORATION", b"\x10" offset = 35
        signature = b"COPYRIGHT BY SNK CORPORATION", b"\x10" offset = 35
    ]
    name = "Neo Geo Pocket Color ROM"
    short_name = "Neo Geo Pocket Color ROM"
    mime = "application/x-neo-geo-pocket-rom"
    extensions = [
        "ngc",
    ]
    kind = Rom

    format = NeoGeoPocketRom
    signatures = [
        signature = b" LICENSED BY SNK CORPORATION"
        signature = b"COPYRIGHT BY SNK CORPORATION"
    ]
    name = "Neo Geo Pocket ROM"
    short_name = "Neo Geo Pocket ROM"
    mime = "application/x-neo-geo-pocket-rom"
    extensions = [
        "ngp",
    ]
    kind = Rom

    format = NewExecutable
    signatures = []
    name = "New Executable"
    short_name = "New Executable"
    mime = "application/x-ms-ne-executable"
    extensions = [
        "exe",
    ]
    kind = Executable

    format = NikonElectronicFile
    signatures = [
        signature = b"\x49\x49\x2A\x00", b"\x1C\x00\xFE\x00" offset = 8
        signature = b"\x49\x49\x2A\x00", b"\x1F\x00\x0B\x00" offset = 8
        signature = b"\x4D\x4D\x00\x2A", b"\x1C\x00\xFE\x00" offset = 8
        signature = b"\x4D\x4D\x00\x2A", b"\x1F\x00\x0B\x00" offset = 8
    ]
    name = "Nikon Electronic File"
    short_name = "Nikon Electronic File"
    mime = "image/x-nikon-nef"
    extensions = [
        "nef",
    ]
    kind = Image

    format = Nintendo64Rom
    signatures = [
        signature = b"\x12\x40\x80\x37\x00\x0F\x00\x00"
        signature = b"\x37\x80\x40\x12\x00\x00\x0F\x00"
        signature = b"\x40\x12\x37\x80\x0F\x00\x00\x00"
        signature = b"\x80\x37\x12\x40\x00\x00\x00\x0F"
    ]
    name = "Nintendo 64 ROM"
    short_name = "Nintendo 64 ROM"
    mime = "application/x-n64-rom"
    extensions = [
        "z64",
    ]
    kind = Rom

    format = NintendoDsRom
    signatures = [
        signature = b"\x24\xFF\xAE\x51\x69\x9A\xA2\x21" offset = 192
        signature = b"\xC8\x60\x4F\xE2\x01\x70\x8F\xE2" offset = 192
    ]
    name = "Nintendo DS ROM"
    short_name = "Nintendo DS ROM"
    mime = "application/x-nintendo-ds-rom"
    extensions = [
        "nds",
    ]
    kind = Rom

    format = NintendoEntertainmentSystemRom
    signatures = [
        signature = b"NES\x1A"
    ]
    name = "Nintendo Entertainment System ROM"
    short_name = "Nintendo Entertainment System ROM"
    mime = "application/x-nintendo-nes-rom"
    extensions = [
        "nes",
    ]
    kind = Rom

    format = NintendoSwitchExecutable
    signatures = [
        signature = b"NSO0"
    ]
    name = "Nintendo Switch Executable"
    short_name = "Nintendo Switch Executable"
    mime = "application/x-nintendo-switch-executable"
    extensions = [
        "nso",
    ]
    kind = Executable

    format = NintendoSwitchPackage
    signatures = [
        signature = b"PFS0"
    ]
    name = "Nintendo Switch Package"
    short_name = "Nintendo Switch Package"
    mime = "application/x-nintendo-switch-package"
    extensions = [
        "nsp",
    ]
    kind = Package

    format = NintendoSwitchRom
    signatures = [
        signature = b"HEAD" offset = 256, b"\xE0" offset = 269
        signature = b"HEAD" offset = 256, b"\xE1" offset = 269
        signature = b"HEAD" offset = 256, b"\xE2" offset = 269
        signature = b"HEAD" offset = 256, b"\xF0" offset = 269
        signature = b"HEAD" offset = 256, b"\xF8" offset = 269
        signature = b"HEAD" offset = 256, b"\xFA" offset = 269
    ]
    name = "Nintendo Switch ROM"
    short_name = "Nintendo Switch ROM"
    mime = "application/x-nintendo-switch-rom"
    extensions = [
        "xci",
    ]
    kind = Rom

    format = OfficeOpenXmlDocument
    signatures = []
    name = "Office Open XML Document"
    short_name = "Office Open XML Document"
    mime = "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
    extensions = [
        "docx",
    ]
    kind = Document

    format = OfficeOpenXmlDrawing
    signatures = []
    name = "Office Open XML Drawing"
    short_name = "Office Open XML Drawing"
    mime = "application/vnd.ms-visio.drawing.main+xml"
    extensions = [
        "vsdx",
    ]
    kind = Diagram

    format = OfficeOpenXmlPresentation
    signatures = []
    name = "Office Open XML Presentation"
    short_name = "Office Open XML Presentation"
    mime = "application/vnd.openxmlformats-officedocument.presentationml.presentation"
    extensions = [
        "pptx",
    ]
    kind = Presentation

    format = OfficeOpenXmlSpreadsheet
    signatures = []
    name = "Office Open XML Spreadsheet"
    short_name = "Office Open XML Spreadsheet"
    mime = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
    extensions = [
        "xlsx",
    ]
    kind = Spreadsheet

    format = OggFlac
    signatures = [
        signature = b"OggS", b"\x7FFLAC" offset = 28
    ]
    name = "Ogg FLAC"
    short_name = "Ogg FLAC"
    mime = "audio/ogg"
    extensions = [
        "oga",
    ]
    kind = Audio

    format = OggMedia
    signatures = [
        signature = b"OggS", b"\x01video" offset = 28
    ]
    name = "Ogg Media"
    short_name = "Ogg Media"
    mime = "video/ogg"
    extensions = [
        "ogm",
    ]
    kind = Video

    format = OggMultiplexedMedia
    signatures = [
        signature = b"OggS"
    ]
    name = "Ogg Multiplexed Media"
    short_name = "Ogg Multiplexed Media"
    mime = "application/ogg"
    extensions = [
        "ogx",
    ]
    kind = Other

    format = OggOpus
    signatures = [
        signature = b"OggS", b"OpusHead" offset = 28
    ]
    name = "Ogg Opus"
    short_name = "Ogg Opus"
    mime = "audio/opus"
    extensions = [
        "opus",
    ]
    kind = Audio

    format = OggSpeex
    signatures = [
        signature = b"OggS", b"Speex  " offset = 28
    ]
    name = "Ogg Speex"
    short_name = "Ogg Speex"
    mime = "audio/ogg"
    extensions = [
        "spx",
    ]
    kind = Audio

    format = OggTheora
    signatures = [
        signature = b"OggS", b"\x80theora" offset = 28
    ]
    name = "Ogg Theora"
    short_name = "Ogg Theora"
    mime = "video/ogg"
    extensions = [
        "ogv",
    ]
    kind = Video

    format = OggVorbis
    signatures = [
        signature = b"OggS", b"\x01vorbis" offset = 28
    ]
    name = "Ogg Vorbis"
    short_name = "Ogg Vorbis"
    mime = "audio/ogg"
    extensions = [
        "ogg",
    ]
    kind = Audio

    format = OlympusRawFormat
    signatures = [
        signature = b"\x49\x49\x52\x4F\x08\x00\x00\x00\x18"
    ]
    name = "Olympus Raw Format"
    short_name = "Olympus Raw Format"
    mime = "image/x-olympus-orf"
    extensions = [
        "orf",
    ]
    kind = Image

    format = OpendocumentDatabase
    signatures = []
    name = "OpenDocument Database"
    short_name = "OpenDocument Database"
    mime = "application/vnd.oasis.opendocument.database"
    extensions = [
        "odb",
    ]
    kind = Database

    format = OpendocumentFormula
    signatures = []
    name = "OpenDocument Formula"
    short_name = "OpenDocument Formula"
    mime = "application/vnd.oasis.opendocument.formula"
    extensions = [
        "odf",
    ]
    kind = Formula

    format = OpendocumentFormulaTemplate
    signatures = []
    name = "OpenDocument Formula Template"
    short_name = "OpenDocument Formula Template"
    mime = "application/vnd.oasis.opendocument.formula-template"
    extensions = [
        "otf",
    ]
    kind = Formula

    format = OpendocumentGraphics
    signatures = []
    name = "OpenDocument Graphics"
    short_name = "OpenDocument Graphics"
    mime = "application/vnd.oasis.opendocument.graphics"
    extensions = [
        "odg",
    ]
    kind = Image

    format = OpendocumentGraphicsTemplate
    signatures = []
    name = "OpenDocument Graphics Template"
    short_name = "OpenDocument Graphics Template"
    mime = "application/vnd.oasis.opendocument.graphics-template"
    extensions = [
        "otg",
    ]
    kind = Image

    format = OpendocumentPresentation
    signatures = []
    name = "OpenDocument Presentation"
    short_name = "OpenDocument Presentation"
    mime = "application/vnd.oasis.opendocument.presentation"
    extensions = [
        "odp",
    ]
    kind = Presentation

    format = OpendocumentPresentationTemplate
    signatures = []
    name = "OpenDocument Presentation Template"
    short_name = "OpenDocument Presentation Template"
    mime = "application/vnd.oasis.opendocument.presentation-template"
    extensions = [
        "otp",
    ]
    kind = Presentation

    format = OpendocumentSpreadsheet
    signatures = []
    name = "OpenDocument Spreadsheet"
    short_name = "OpenDocument Spreadsheet"
    mime = "application/vnd.oasis.opendocument.spreadsheet"
    extensions = [
        "ods",
    ]
    kind = Spreadsheet

    format = OpendocumentSpreadsheetTemplate
    signatures = []
    name = "OpenDocument Spreadsheet Template"
    short_name = "OpenDocument Spreadsheet Template"
    mime = "application/vnd.oasis.opendocument.spreadsheet-template"
    extensions = [
        "ots",
    ]
    kind = Spreadsheet

    format = OpendocumentText
    signatures = []
    name = "OpenDocument Text"
    short_name = "OpenDocument Text"
    mime = "application/vnd.oasis.opendocument.text"
    extensions = [
        "odt",
    ]
    kind = Document

    format = OpendocumentTextMaster
    signatures = []
    name = "OpenDocument Text Master"
    short_name = "OpenDocument Text Master"
    mime = "application/vnd.oasis.opendocument.text-master"
    extensions = [
        "odm",
    ]
    kind = Document

    format = OpendocumentTextMasterTemplate
    signatures = []
    name = "OpenDocument Text Master Template"
    short_name = "OpenDocument Text Master Template"
    mime = "application/vnd.oasis.opendocument.text-master-template"
    extensions = [
        "otm",
    ]
    kind = Document

    format = OpendocumentTextTemplate
    signatures = []
    name = "OpenDocument Text Template"
    short_name = "OpenDocument Text Template"
    mime = "application/vnd.oasis.opendocument.text-template"
    extensions = [
        "ott",
    ]
    kind = Document

    format = Openexr
    signatures = [
        signature = b"\x76\x2F\x31\x01"
    ]
    name = "OpenEXR"
    short_name = "OpenEXR"
    mime = "image/x-exr"
    extensions = [
        "exr",
    ]
    kind = Image

    format = Opennurbs
    signatures = [
        signature = b"3D Geometry File Format"
    ]
    name = "openNURBS"
    short_name = "openNURBS"
    mime = "model/x-3dm"
    extensions = [
        "3dm",
    ]
    kind = Model

    format = Openraster
    signatures = []
    name = "OpenRaster"
    short_name = "OpenRaster"
    mime = "image/openraster"
    extensions = [
        "ora",
    ]
    kind = Image

    format = Opentype
    signatures = [
        signature = b"OTTO\0"
    ]
    name = "OpenType"
    short_name = "OpenType"
    mime = "font/otf"
    extensions = [
        "otf",
    ]
    kind = Font

    format = Openxps
    signatures = []
    name = "OpenXPS"
    short_name = "OpenXPS"
    mime = "application/oxps"
    extensions = [
        "oxps",
    ]
    kind = Document

    format = OptimizedDalvikExecutable
    signatures = [
        signature = b"dey\n"
    ]
    name = "Optimized Dalvik Executable"
    short_name = "Optimized Dalvik Executable"
    mime = "application/vnd.android.dey"
    extensions = [
        "dey",
    ]
    kind = Executable

    format = PanasonicRaw
    signatures = [
        signature = b"\x49\x49\x55\x00\x18\x00\x00\x00\x88\xE7\x74\xD8"
    ]
    name = "Panasonic Raw"
    short_name = "Panasonic Raw"
    mime = "image/x-panasonic-rw2"
    extensions = [
        "rw2",
    ]
    kind = Image

    format = PcapDump
    signatures = [
        signature = b"\xA1\xB2\xC3\xD4"
        signature = b"\xD4\xC3\xB2\xA1"
    ]
    name = "PCAP Dump"
    short_name = "PCAP Dump"
    mime = "application/vnd.tcpdump.pcap"
    extensions = [
        "pcap",
    ]
    kind = Other

    format = PcapNextGenerationDump
    signatures = [
        signature = b"\x0A\x0D\x0D\x0A"
    ]
    name = "PCAP Next Generation Dump"
    short_name = "PCAP Next Generation Dump"
    mime = "application/x-pcapng"
    extensions = [
        "pcapng",
    ]
    kind = Other

    format = PemCertificate
    signatures = [
        signature = b"-----BEGIN CERTIFICATE-----"
    ]
    name = "PEM Certificate"
    short_name = "PEM Certificate"
    mime = "application/x-pem-file"
    extensions = [
        "crt",
    ]
    kind = Other

    format = PemCertificateSigningRequest
    signatures = [
        signature = b"-----BEGIN CERTIFICATE REQUEST-----"
    ]
    name = "PEM Certificate Signing Request"
    short_name = "PEM Certificate Signing Request"
    mime = "application/x-pem-file"
    extensions = [
        "csr",
    ]
    kind = Other

    format = PemPrivateKey
    signatures = [
        signature = b"-----BEGIN ENCRYPTED PRIVATE KEY-----"
        signature = b"-----BEGIN ECDSA PRIVATE KEY-----"
        signature = b"-----BEGIN DSA PRIVATE KEY-----"
        signature = b"-----BEGIN RSA PRIVATE KEY-----"
        signature = b"-----BEGIN EC PRIVATE KEY-----"
        signature = b"-----BEGIN PRIVATE KEY-----"
    ]
    name = "PEM Private Key"
    short_name = "PEM Private Key"
    mime = "application/x-pem-file"
    extensions = [
        "key",
    ]
    kind = Other

    format = PemPublicKey
    signatures = [
        signature = b"-----BEGIN ECDSA PUBLIC KEY-----"
        signature = b"-----BEGIN DSA PUBLIC KEY-----"
        signature = b"-----BEGIN RSA PUBLIC KEY-----"
        signature = b"-----BEGIN EC PUBLIC KEY-----"
        signature = b"-----BEGIN PUBLIC KEY-----"
    ]
    name = "PEM Public Key"
    short_name = "PEM Public Key"
    mime = "application/x-pem-file"
    extensions = [
        "pub",
    ]
    kind = Other

    format = PerlScript
    signatures = [
        signature = b"#!/usr/bin/env perl"
        signature = b"#!/usr/bin/perl"
    ]
    name = "Perl Script"
    short_name = "Perl Script"
    mime = "text/x-perl"
    extensions = [
        "pl",
    ]
    kind = Other

    format = PersonalStorageTable
    signatures = [
        signature = b"!BDN"
    ]
    name = "Personal Storage Table"
    short_name = "Personal Storage Table"
    mime = "application/vnd.ms-outlook"
    extensions = [
        "pst",
    ]
    kind = Other

    format = PgpMessage
    signatures = [
        signature = b"-----BEGIN PGP MESSAGE-----"
    ]
    name = "PGP Message"
    short_name = "PGP Message"
    mime = "application/pgp"
    extensions = [
        "asc",
    ]
    kind = Other

    format = PgpPrivateKeyBlock
    signatures = [
        signature = b"-----BEGIN PGP PRIVATE KEY BLOCK-----"
    ]
    name = "PGP Private Key Block"
    short_name = "PGP Private Key Block"
    mime = "application/pgp-keys"
    extensions = [
        "asc",
    ]
    kind = Other

    format = PgpPublicKeyBlock
    signatures = [
        signature = b"-----BEGIN PGP PUBLIC KEY BLOCK-----"
    ]
    name = "PGP Public Key Block"
    short_name = "PGP Public Key Block"
    mime = "application/pgp-keys"
    extensions = [
        "asc",
    ]
    kind = Other

    format = PgpSignature
    signatures = [
        signature = b"-----BEGIN PGP SIGNATURE-----"
    ]
    name = "PGP Signature"
    short_name = "PGP Signature"
    mime = "application/pgp-signature"
    extensions = [
        "asc",
    ]
    kind = Other

    format = PgpSignedMessage
    signatures = [
        signature = b"-----BEGIN PGP SIGNED MESSAGE-----"
    ]
    name = "PGP Signed Message"
    short_name = "PGP Signed Message"
    mime = "application/pgp"
    extensions = [
        "asc",
    ]
    kind = Other

    format = PictureExchange
    signatures = [
        signature = b"\x0A\x00\x00", b"\x00" offset = 64
        signature = b"\x0A\x00\x01", b"\x00" offset = 64
        signature = b"\x0A\x02\x00", b"\x00" offset = 64
        signature = b"\x0A\x02\x01", b"\x00" offset = 64
        signature = b"\x0A\x03\x00", b"\x00" offset = 64
        signature = b"\x0A\x03\x01", b"\x00" offset = 64
        signature = b"\x0A\x04\x00", b"\x00" offset = 64
        signature = b"\x0A\x04\x01", b"\x00" offset = 64
        signature = b"\x0A\x05\x00", b"\x00" offset = 64
        signature = b"\x0A\x05\x01", b"\x00" offset = 64
    ]
    name = "Picture Exchange"
    short_name = "Picture Exchange"
    mime = "image/x-pcx"
    extensions = [
        "pcx",
    ]
    kind = Image

    format = PlainText
    signatures = []
    name = "Plain Text"
    short_name = "Plain Text"
    mime = "text/plain"
    extensions = [
        "txt",
    ]
    kind = Other

    format = Pmarc
    signatures = [
        signature = b"-pc1-" offset = 2
        signature = b"-pm0-" offset = 2
        signature = b"-pm1-" offset = 2
        signature = b"-pm2-" offset = 2
        signature = b"-pms-" offset = 2
    ]
    name = "PMarc"
    short_name = "PMarc"
    mime = "application/x-lzh-compressed"
    extensions = [
        "pma",
    ]
    kind = Archive

    format = PMTiles
    signatures = [
        signature = b"PMTiles"
    ]
    name = "PMTiles"
    short_name = "PMTiles"
    mime = "application/vnd.pmtiles"
    extensions = [
        "pmtiles"
    ]
    kind = Geospatial

    format = PolygonAscii
    signatures = [
        signature = b"ply\r\nformat ascii"
        signature = b"ply\nformat ascii"
        signature = b"ply\rformat ascii"
    ]
    name = "Polygon ASCII"
    short_name = "Polygon ASCII"
    mime = "model/x-ply-ascii"
    extensions = [
        "ply",
    ]
    kind = Model

    format = PolygonBinary
    signatures = [
        signature = b"ply\r\nformat binary_little_endian"
        signature = b"ply\nformat binary_little_endian"
        signature = b"ply\rformat binary_little_endian"
        signature = b"ply\r\nformat binary_big_endian"
        signature = b"ply\nformat binary_big_endian"
        signature = b"ply\rformat binary_big_endian"
    ]
    name = "Polygon Binary"
    short_name = "Polygon Binary"
    mime = "model/x-ply-binary"
    extensions = [
        "ply",
    ]
    kind = Model

    format = PortableArbitraryMap
    signatures = [
        signature = b"P7 "
        signature = b"P7\n"
        signature = b"P7\r"
        signature = b"P7\t"
    ]
    name = "Portable Arbitrary Map"
    short_name = "Portable Arbitrary Map"
    mime = "image/x-portable-arbitrarymap"
    extensions = [
        "pam",
    ]
    kind = Image

    format = PortableBitmap
    signatures = [
        signature = b"P1 "
        signature = b"P1\n"
        signature = b"P1\r"
        signature = b"P1\t"
        signature = b"P4 "
        signature = b"P4\n"
        signature = b"P4\r"
        signature = b"P4\t"
    ]
    name = "Portable BitMap"
    short_name = "Portable BitMap"
    mime = "image/x-portable-bitmap"
    extensions = [
        "pbm",
    ]
    kind = Image

    format = PortableDocumentFormat
    signatures = [
        signature = b"%PDF-"
    ]
    name = "Portable Document Format"
    short_name = "Portable Document Format"
    mime = "application/pdf"
    extensions = [
        "pdf",
    ]
    kind = Document

    format = PortableExecutable
    signatures = []
    name = "Portable Executable"
    short_name = "Portable Executable"
    mime = "application/vnd.microsoft.portable-executable"
    extensions = [
        "exe",
    ]
    kind = Executable

    format = PortableFloatmap
    signatures = [
        signature = b"PF "
        signature = b"PF\n"
        signature = b"PF\r"
        signature = b"PF\t"
        signature = b"Pf "
        signature = b"Pf\n"
        signature = b"Pf\r"
        signature = b"Pf\t"
    ]
    name = "Portable FloatMap"
    short_name = "Portable FloatMap"
    mime = "image/x-pfm"
    extensions = [
        "pfm",
    ]
    kind = Image

    format = PortableGraymap
    signatures = [
        signature = b"P2 "
        signature = b"P2\n"
        signature = b"P2\r"
        signature = b"P2\t"
        signature = b"P5 "
        signature = b"P5\n"
        signature = b"P5\r"
        signature = b"P5\t"
    ]
    name = "Portable GrayMap"
    short_name = "Portable GrayMap"
    mime = "image/x-portable-graymap"
    extensions = [
        "pgm",
    ]
    kind = Image

    format = PortableNetworkGraphics
    signatures = [
        signature = b"\x89PNG\r\n\x1A\n"
    ]
    name = "Portable Network Graphics"
    short_name = "Portable Network Graphics"
    mime = "image/png"
    extensions = [
        "png",
    ]
    kind = Image

    format = PortablePixmap
    signatures = [
        signature = b"P3 "
        signature = b"P3\n"
        signature = b"P3\r"
        signature = b"P3\t"
        signature = b"P6 "
        signature = b"P6\n"
        signature = b"P6\r"
        signature = b"P6\t"
    ]
    name = "Portable PixMap"
    short_name = "Portable PixMap"
    mime = "image/x-portable-pixmap"
    extensions = [
        "ppm",
    ]
    kind = Image

    format = Postscript
    signatures = [
        signature = b"%!"
    ]
    name = "PostScript"
    short_name = "PostScript"
    mime = "application/postscript"
    extensions = [
        "ps",
    ]
    kind = Document

    format = PythonScript
    signatures = [
        signature = b"#!/usr/local/bin/python"
        signature = b"#!/usr/bin/env python"
        signature = b"#!/usr/bin/python"
    ]
    name = "Python Script"
    short_name = "Python Script"
    mime = "text/x-python"
    extensions = [
        "py",
    ]
    kind = Other

    format = QemuCopyOnWrite
    signatures = [
        signature = b"QFI\xFB"
    ]
    name = "QEMU Copy On Write"
    short_name = "QEMU Copy On Write"
    mime = "application/x-qemu-disk"
    extensions = [
        "qcow",
    ]
    kind = Disk

    format = QualcommPurevoice
    signatures = [
        signature = b"RIFF", b"QLCM" offset = 8
    ]
    name = "Qualcomm PureVoice"
    short_name = "Qualcomm PureVoice"
    mime = "audio/qcelp"
    extensions = [
        "qcp",
    ]
    kind = Audio

    format = QuiteOkAudio
    signatures = [
        signature = b"qoaf"
    ]
    name = "Quite OK Audio"
    short_name = "Quite OK Audio"
    mime = "audio/x-qoa"
    extensions = [
        "qoa",
    ]
    kind = Audio

    format = QuiteOkImage
    signatures = [
        signature = b"qoif"
    ]
    name = "Quite OK Image"
    short_name = "Quite OK Image"
    mime = "image/x-qoi"
    extensions = [
        "qoi",
    ]
    kind = Image

    format = RadianceHdr
    signatures = [
        signature = b"#?RADIANCE\n"
    ]
    name = "Radiance HDR"
    short_name = "Radiance HDR"
    mime = "image/vnd.radiance"
    extensions = [
        "hdr",
    ]
    kind = Image

    format = Realaudio
    signatures = [
        signature = b".ra\xFD"
    ]
    name = "RealAudio"
    short_name = "RealAudio"
    mime = "audio/x-pn-realaudio"
    extensions = [
        "ra",
    ]
    kind = Audio

    format = ReallySimpleSyndication
    signatures = [
        signature = b"\xEF\xBB\xBF<rss"
        signature = b"<rss"
    ]
    name = "Really Simple Syndication"
    short_name = "Really Simple Syndication"
    mime = "application/rss+xml"
    extensions = [
        "rss",
    ]
    kind = Other

    format = Realmedia
    signatures = [
        signature = b".RMF\0\0\0\x12"
    ]
    name = "RealMedia"
    short_name = "RealMedia"
    mime = "application/vnd.rn-realmedia"
    extensions = [
        "rm",
    ]
    kind = Other

    format = Realvideo
    signatures = []
    name = "RealVideo"
    short_name = "RealVideo"
    mime = "video/x-pn-realvideo"
    extensions = [
        "rv",
    ]
    kind = Video

    format = RedHatPackageManager
    signatures = [
        signature = b"\xED\xAB\xEE\xDB"
    ]
    name = "Red Hat Package Manager"
    short_name = "Red Hat Package Manager"
    mime = "application/x-rpm"
    extensions = [
        "rpm",
    ]
    kind = Package

    format = RichTextFormat
    signatures = [
        signature = b"{\\rtf"
    ]
    name = "Rich Text Format"
    short_name = "Rich Text Format"
    mime = "application/rtf"
    extensions = [
        "rtf",
    ]
    kind = Document

    format = RoshalArchive
    signatures = [
        signature = b"Rar!\x1A\x07\x01\0"
        signature = b"Rar!\x1A\x07\0"
    ]
    name = "Roshal Archive"
    short_name = "Roshal Archive"
    mime = "application/vnd.rar"
    extensions = [
        "rar",
    ]
    kind = Archive

    format = RubyScript
    signatures = [
        signature = b"#!/usr/local/bin/ruby"
        signature = b"#!/usr/bin/env ruby"
        signature = b"#!/usr/bin/ruby"
    ]
    name = "Ruby Script"
    short_name = "Ruby Script"
    mime = "text/x-ruby"
    extensions = [
        "rb",
    ]
    kind = Other

    format = Rzip
    signatures = [
        signature = b"RZIP"
    ]
    name = "rzip"
    short_name = "rzip"
    mime = "application/x-rzip"
    extensions = [
        "rz",
    ]
    kind = Compressed

    format = ScalableVectorGraphics
    signatures = [
        signature = b"\xEF\xBB\xBF<SVG"
        signature = b"\xEF\xBB\xBF<svg"
        signature = b"<SVG"
        signature = b"<svg"
    ]
    name = "Scalable Vector Graphics"
    short_name = "Scalable Vector Graphics"
    mime = "image/svg+xml"
    extensions = [
        "svg",
    ]
    kind = Image

    format = ScreamTracker3Module
    signatures = [
        signature = b"SCRM" offset = 44
    ]
    name = "Scream Tracker 3 Module"
    short_name = "Scream Tracker 3 Module"
    mime = "audio/x-s3m"
    extensions = [
        "s3m",
    ]
    kind = Audio

    format = SegaMasterSystemRom
    signatures = [
        signature = b"TMR SEGA" offset = 32752, b"\x30" offset = 32767
        signature = b"TMR SEGA" offset = 32752, b"\x31" offset = 32767
        signature = b"TMR SEGA" offset = 32752, b"\x3C" offset = 32767
        signature = b"TMR SEGA" offset = 32752, b"\x3E" offset = 32767
        signature = b"TMR SEGA" offset = 32752, b"\x3F" offset = 32767
        signature = b"TMR SEGA" offset = 32752, b"\x40" offset = 32767
        signature = b"TMR SEGA" offset = 32752, b"\x41" offset = 32767
        signature = b"TMR SEGA" offset = 32752, b"\x4C" offset = 32767
        signature = b"TMR SEGA" offset = 32752, b"\x4E" offset = 32767
        signature = b"TMR SEGA" offset = 32752, b"\x4F" offset = 32767
    ]
    name = "Sega Master System ROM"
    short_name = "Sega Master System ROM"
    mime = "application/x-sms-rom"
    extensions = [
        "sms",
    ]
    kind = Rom

    format = Seqbox
    signatures = [
        signature = b"SBx"
    ]
    name = "SeqBox"
    short_name = "SeqBox"
    mime = "application/x-sbx"
    extensions = [
        "sbx",
    ]
    kind = Archive

    format = SevenZip
    signatures = [
        signature = b"7z\xBC\xAF\x27\x1C"
    ]
    name = "7-Zip"
    short_name = "7-Zip"
    mime = "application/x-7z-compressed"
    extensions = [
        "7z",
    ]
    kind = Archive

    format = Shapefile
    signatures = [
        signature = b"\0\0'\n"
    ]
    name = "Shapefile"
    short_name = "Shapefile"
    mime = "application/x-esri-shape"
    extensions = [
        "shp",
    ]
    kind = Geospatial

    format = ShellScript
    signatures = [
        signature = b"#!/usr/local/bin/bash"
        signature = b"#!/usr/local/bin/fish"
        signature = b"#!/usr/local/bin/tcsh"
        signature = b"#!/usr/local/bin/ash"
        signature = b"#!/usr/local/bin/zsh"
        signature = b"#!/usr/bin/env bash"
        signature = b"#!/usr/bin/env fish"
        signature = b"#!/usr/bin/env zsh"
        signature = b"#!/usr/local/bash"
        signature = b"#!/usr/local/tcsh"
        signature = b"#!/usr/bin/bash"
        signature = b"#!/usr/bin/fish"
        signature = b"#!/usr/bin/tcsh"
        signature = b"#!/usr/bin/zsh"
        signature = b"#!/bin/bash"
        signature = b"#!/bin/tcsh"
        signature = b"#!/bin/ash"
        signature = b"#!/bin/csh"
        signature = b"#!/bin/ksh"
        signature = b"#!/bin/zsh"
        signature = b"#!/bin/sh"
    ]
    name = "Shell Script"
    short_name = "Shell Script"
    mime = "text/x-shellscript"
    extensions = [
        "sh",
    ]
    kind = Other

    format = ShoutcastPlaylist
    signatures = [
        signature = b"[playlist]\r\n"
        signature = b"[playlist]\n"
        signature = b"[playlist]\r"
    ]
    name = "SHOUTcast Playlist"
    short_name = "SHOUTcast Playlist"
    mime = "audio/x-scpls"
    extensions = [
        "pls",
    ]
    kind = Playlist

    format = SiliconGraphicsImage
    signatures = [
        signature = b"\x01\xDA"
    ]
    name = "Silicon Graphics Image"
    short_name = "Silicon Graphics Image"
    mime = "image/x-sgi"
    extensions = [
        "sgi",
    ]
    kind = Image

    format = SiliconGraphicsMovie
    signatures = [
        signature = b"MOVI"
    ]
    name = "Silicon Graphics Movie"
    short_name = "Silicon Graphics Movie"
    mime = "video/x-sgi-movie"
    extensions = [
        "movie",
    ]
    kind = Video

    format = SimpleObjectAccessProtocol
    signatures = [
        signature = b"\xEF\xBB\xBF<soap"
        signature = b"<soap"
    ]
    name = "Simple Object Access Protocol"
    short_name = "Simple Object Access Protocol"
    mime = "application/soap+xml"
    extensions = [
        "soap",
    ]
    kind = Other

    format = Sketch
    signatures = []
    name = "Sketch"
    short_name = "Sketch"
    mime = "image/x-sketch"
    extensions = [
        "sketch",
    ]
    kind = Image

    format = Sketch43
    signatures = []
    name = "Sketch 43"
    short_name = "Sketch 43"
    mime = "image/x-sketch"
    extensions = [
        "sketch",
    ]
    kind = Image

    format = Sketchup
    signatures = [
        signature = b"\xFF\xFE\xFF\x0ES\0k\0e\0t\0c\0h\0U\0p\0 \0M\0o\0d\0e\0l\0"
    ]
    name = "SketchUp"
    short_name = "SketchUp"
    mime = "application/vnd.sketchup.skp"
    extensions = [
        "skp",
    ]
    kind = Model

    format = SmallWebFormat
    signatures = [
        signature = b"CWS"
        signature = b"FWS"
        signature = b"ZWS"
    ]
    name = "Small Web Format"
    short_name = "Small Web Format"
    mime = "application/x-shockwave-flash"
    extensions = [
        "swf",
    ]
    kind = Other

    format = Snappy
    signatures = [
        signature = b"\xFF\x06\0\0sNaPpY"
    ]
    name = "Snappy"
    short_name = "Snappy"
    mime = "application/x-snappy-framed"
    extensions = [
        "sz",
    ]
    kind = Compressed

    format = SolidworksAssembly
    signatures = []
    name = "SolidWorks Assembly"
    short_name = "SolidWorks Assembly"
    mime = "model/x-sldasm"
    extensions = [
        "sldasm",
    ]
    kind = Model

    format = SolidworksDrawing
    signatures = []
    name = "SolidWorks Drawing"
    short_name = "SolidWorks Drawing"
    mime = "model/x-slddrw"
    extensions = [
        "slddrw",
    ]
    kind = Model

    format = SolidworksPart
    signatures = []
    name = "SolidWorks Part"
    short_name = "SolidWorks Part"
    mime = "model/x-sldprt"
    extensions = [
        "sldprt",
    ]
    kind = Model

    format = SonyAlphaRaw
    signatures = [
        signature = b"\x49\x49\x2A\x00", b"\x08\x00\x00\x00" offset = 4, b"\xFE\x00\x04\x00\x01\x00\x00\x00\x01\x00\x00\x00" offset = 10
    ]
    name = "Sony Alpha Raw"
    short_name = "Sony Alpha Raw"
    mime = "image/x-sony-arw"
    extensions = [
        "arw",
    ]
    kind = Image

    format = SonyDsdStreamFile
    signatures = [
        signature = b"DSD "
    ]
    name = "Sony DSD Stream File"
    short_name = "Sony DSD Stream File"
    mime = "audio/x-dsf"
    extensions = [
        "dsf",
    ]
    kind = Audio

    format = SonyMovie
    signatures = [
        signature = b"ftypmqt" offset = 4
    ]
    name = "Sony Movie"
    short_name = "Sony Movie"
    mime = "video/quicktime"
    extensions = [
        "mqv",
    ]
    kind = Video

    format = Soundfont2
    signatures = [
        signature = b"RIFF", b"sfbk" offset = 8
    ]
    name = "SoundFont 2"
    short_name = "SoundFont 2"
    mime = "audio/x-soundfont"
    extensions = [
        "sf2",
    ]
    kind = Audio

    format = SpaceclaimDocument
    signatures = []
    name = "SpaceClaim Document"
    short_name = "SpaceClaim Document"
    mime = "model/x-scdoc"
    extensions = [
        "scdoc",
    ]
    kind = Model

    format = Sqlite3
    signatures = [
        signature = b"SQLite format 3\0"
    ]
    name = "SQLite 3"
    short_name = "SQLite 3"
    mime = "application/vnd.sqlite3"
    extensions = [
        "sqlite",
    ]
    kind = Database

    format = Squashfs
    signatures = [
        signature = b"hsqs"
    ]
    name = "squashfs"
    short_name = "squashfs"
    mime = "application/x-squashfs"
    extensions = [
        "sqsh",
    ]
    kind = Archive

    format = StandardForTheExchangeOfProductModelData
    signatures = [
        signature = b"ISO-10303-21;"
    ]
    name = "Standard for the Exchange of Product model data"
    short_name = "Standard for the Exchange of Product model data"
    mime = "model/step"
    extensions = [
        "step",
    ]
    kind = Model

    format = Starcalc
    signatures = []
    name = "StarCalc"
    short_name = "StarCalc"
    mime = "application/vnd.stardivision.calc"
    extensions = [
        "sdc",
    ]
    kind = Spreadsheet

    format = Starchart
    signatures = []
    name = "StarChart"
    short_name = "StarChart"
    mime = "application/vnd.stardivision.chart"
    extensions = [
        "sds",
    ]
    kind = Diagram

    format = Stardraw
    signatures = []
    name = "StarDraw"
    short_name = "StarDraw"
    mime = "application/vnd.stardivision.draw"
    extensions = [
        "sda",
    ]
    kind = Image

    format = Starimpress
    signatures = []
    name = "StarImpress"
    short_name = "StarImpress"
    mime = "application/vnd.stardivision.impress"
    extensions = [
        "sdd",
    ]
    kind = Presentation

    format = Starmath
    signatures = []
    name = "StarMath"
    short_name = "StarMath"
    mime = "application/vnd.stardivision.math"
    extensions = [
        "smf",
    ]
    kind = Formula

    format = Starwriter
    signatures = []
    name = "StarWriter"
    short_name = "StarWriter"
    mime = "application/vnd.stardivision.writer"
    extensions = [
        "sdw",
    ]
    kind = Document

    format = StereolithographyAscii
    signatures = [
        signature = b"solid "
    ]
    name = "Stereolithography ASCII"
    short_name = "Stereolithography ASCII"
    mime = "model/x-stl-ascii"
    extensions = [
        "stl",
    ]
    kind = Model

    format = Stuffit
    signatures = [
        signature = b"StuffIt (c)1997", b"\x05" offset = 82
        signature = b"SIT!" offset = 128, b"rLau" offset = 138
        signature = b"SIT!", b"rLau" offset = 10
    ]
    name = "StuffIt"
    short_name = "StuffIt"
    mime = "application/x-stuffit"
    extensions = [
        "sit",
    ]
    kind = Archive

    format = StuffitX
    signatures = [
        signature = b"StuffIt!"
        signature = b"StuffIt?"
    ]
    name = "StuffIt X"
    short_name = "StuffIt X"
    mime = "application/x-stuffitx"
    extensions = [
        "sitx",
    ]
    kind = Archive

    format = SubripText
    signatures = [
        signature = b"\xEF\xBB\xBF1\r\n00:"
        signature = b"\xEF\xBB\xBF1\n00:"
        signature = b"1\r\n00:"
        signature = b"1\n00:"
    ]
    name = "SubRip Text"
    short_name = "SubRip Text"
    mime = "application/x-subrip"
    extensions = [
        "srt",
    ]
    kind = Subtitle

    format = SunXmlCalc
    signatures = []
    name = "Sun XML Calc"
    short_name = "Sun XML Calc"
    mime = "application/vnd.sun.xml.calc"
    extensions = [
        "sxc",
    ]
    kind = Spreadsheet

    format = SunXmlCalcTemplate
    signatures = []
    name = "Sun XML Calc Template"
    short_name = "Sun XML Calc Template"
    mime = "application/vnd.sun.xml.calc.template"
    extensions = [
        "stc",
    ]
    kind = Spreadsheet

    format = SunXmlDraw
    signatures = []
    name = "Sun XML Draw"
    short_name = "Sun XML Draw"
    mime = "application/vnd.sun.xml.draw"
    extensions = [
        "sxd",
    ]
    kind = Image

    format = SunXmlDrawTemplate
    signatures = []
    name = "Sun XML Draw Template"
    short_name = "Sun XML Draw Template"
    mime = "application/vnd.sun.xml.draw.template"
    extensions = [
        "std",
    ]
    kind = Image

    format = SunXmlImpress
    signatures = []
    name = "Sun XML Impress"
    short_name = "Sun XML Impress"
    mime = "application/vnd.sun.xml.impress"
    extensions = [
        "sxi",
    ]
    kind = Presentation

    format = SunXmlImpressTemplate
    signatures = []
    name = "Sun XML Impress Template"
    short_name = "Sun XML Impress Template"
    mime = "application/vnd.sun.xml.impress.template"
    extensions = [
        "sti",
    ]
    kind = Presentation

    format = SunXmlMath
    signatures = []
    name = "Sun XML Math"
    short_name = "Sun XML Math"
    mime = "application/vnd.sun.xml.math"
    extensions = [
        "sxm",
    ]
    kind = Formula

    format = SunXmlWriter
    signatures = []
    name = "Sun XML Writer"
    short_name = "Sun XML Writer"
    mime = "application/vnd.sun.xml.writer"
    extensions = [
        "sxw",
    ]
    kind = Document

    format = SunXmlWriterGlobal
    signatures = []
    name = "Sun XML Writer Global"
    short_name = "Sun XML Writer Global"
    mime = "application/vnd.sun.xml.writer.global"
    extensions = [
        "sgw",
    ]
    kind = Document

    format = SunXmlWriterTemplate
    signatures = []
    name = "Sun XML Writer Template"
    short_name = "Sun XML Writer Template"
    mime = "application/vnd.sun.xml.writer.template"
    extensions = [
        "stw",
    ]
    kind = Document

    format = TagImageFileFormat
    signatures = [
        signature = b"\x49\x49\x2A\x00"
        signature = b"\x49\x49\x2B\x00"
        signature = b"\x4D\x4D\x00\x2A"
        signature = b"\x4D\x4D\x00\x2B"
    ]
    name = "Tag Image File Format"
    short_name = "Tag Image File Format"
    mime = "image/tiff"
    extensions = [
        "tiff",
    ]
    kind = Image

    format = TapeArchive
    signatures = [
        signature = b"ustar\0\x30\x30" offset = 257
        signature = b"ustar  \0" offset = 257
    ]
    name = "Tape Archive"
    short_name = "Tape Archive"
    mime = "application/x-tar"
    extensions = [
        "tar",
    ]
    kind = Archive

    format = Tasty
    signatures = [
        signature = b"\x5C\xA1\xAB\x1F"
    ]
    name = "TASTy"
    short_name = "TASTy"
    mime = "application/x-tasty"
    extensions = [
        "tasty",
    ]
    kind = Metadata

    format = ThirdGenerationPartnershipProject
    signatures = [
        signature = b"ftyp3ge" offset = 4
        signature = b"ftyp3gf" offset = 4
        signature = b"ftyp3gg" offset = 4
        signature = b"ftyp3gh" offset = 4
        signature = b"ftyp3gm" offset = 4
        signature = b"ftyp3gp" offset = 4
        signature = b"ftyp3gr" offset = 4
        signature = b"ftyp3gs" offset = 4
        signature = b"ftyp3gt" offset = 4
    ]
    name = "3rd Generation Partnership Project"
    short_name = "3rd Generation Partnership Project"
    mime = "video/3gpp"
    extensions = [
        "3gp",
    ]
    kind = Video

    format = ThirdGenerationPartnershipProject2
    signatures = [
        signature = b"ftypKDDI" offset = 4
        signature = b"ftyp3g2" offset = 4
    ]
    name = "3rd Generation Partnership Project 2"
    short_name = "3rd Generation Partnership Project 2"
    mime = "video/3gpp2"
    extensions = [
        "3g2",
    ]
    kind = Video

    format = ThreeDimensionalManufacturingFormat
    signatures = []
    name = "3D Manufacturing Format"
    short_name = "3D Manufacturing Format"
    mime = "application/vnd.ms-package.3dmanufacturing-3dmodel+xml"
    extensions = [
        "3mf",
    ]
    kind = Model

    format = ThreeDimensionalStudio
    signatures = [
        signature = b"MM", b"\x02" offset = 6, b"\n" offset = 8, b"==" offset = 16
    ]
    name = "3D Studio"
    short_name = "3D Studio"
    mime = "application/x-3ds"
    extensions = [
        "3ds",
    ]
    kind = Model

    format = ThreeDimensionalStudioMax
    signatures = []
    name = "3D Studio Max"
    short_name = "3D Studio Max"
    mime = "application/x-max"
    extensions = [
        "max",
    ]
    kind = Model

    format = TiledMapXml
    signatures = [
        signature = b"\xEF\xBB\xBF<map"
        signature = b"<map"
    ]
    name = "Tiled Map XML"
    short_name = "Tiled Map XML"
    mime = "application/x-tmx+xml"
    extensions = [
        "tmx",
    ]
    kind = Other

    format = TiledTilesetXml
    signatures = [
        signature = b"\xEF\xBB\xBF<tileset"
        signature = b"<tileset"
    ]
    name = "Tiled Tileset XML"
    short_name = "Tiled Tileset XML"
    mime = "application/x-tsx+xml"
    extensions = [
        "tsx",
    ]
    kind = Other

    format = TimedTextMarkupLanguage
    signatures = [
        signature = b"\xEF\xBB\xBF<tt xmlns=\"http://www.w3.org/ns/ttml\""
        signature = b"<tt xmlns=\"http://www.w3.org/ns/ttml\""
    ]
    name = "Timed Text Markup Language"
    short_name = "Timed Text Markup Language"
    mime = "application/ttml+xml"
    extensions = [
        "ttml",
    ]
    kind = Subtitle

    format = ToolCommandLanguageScript
    signatures = [
        signature = b"#!/usr/local/bin/tclsh"
        signature = b"#!/usr/local/bin/wish"
        signature = b"#!/usr/bin/env tclsh"
        signature = b"#!/usr/local/bin/tcl"
        signature = b"#!/usr/bin/env wish"
        signature = b"#!/usr/bin/env tcl"
        signature = b"#!/usr/bin/tclsh"
        signature = b"#!/usr/bin/wish"
        signature = b"#!/usr/bin/tcl"
    ]
    name = "Tool Command Language Script"
    short_name = "Tool Command Language Script"
    mime = "text/x-tcl"
    extensions = [
        "tcl",
    ]
    kind = Other

    format = TrainingCenterXml
    signatures = [
        signature = b"\xEF\xBB\xBF<TrainingCenterDatabase"
        signature = b"<TrainingCenterDatabase"
    ]
    name = "Training Center XML"
    short_name = "Training Center XML"
    mime = "application/vnd.garmin.tcx+xml"
    extensions = [
        "tcx",
    ]
    kind = Geospatial

    format = Truetype
    signatures = [
        signature = b"\x00\x01\x00\x00\x00"
        signature = b"true"
    ]
    name = "TrueType"
    short_name = "TrueType"
    mime = "font/ttf"
    extensions = [
        "ttf",
    ]
    kind = Font

    format = TruetypeCollection
    signatures = [
        signature = b"ttcf\x00"
    ]
    name = "TrueType Collection"
    short_name = "TrueType Collection"
    mime = "font/collection"
    extensions = [
        "ttc",
    ]
    kind = Font

    format = UltimateSoundtrackerModule
    signatures = [
        signature = b"M.K." offset = 1080
    ]
    name = "Ultimate Soundtracker Module"
    short_name = "Ultimate Soundtracker Module"
    mime = "audio/x-mod"
    extensions = [
        "mod",
    ]
    kind = Audio

    format = UniformOfficeFormatPresentation
    signatures = []
    name = "Uniform Office Format Presentation"
    short_name = "Uniform Office Format Presentation"
    mime = "application/vnd.uof.presentation"
    extensions = [
        "uop",
    ]
    kind = Presentation

    format = UniformOfficeFormatSpreadsheet
    signatures = []
    name = "Uniform Office Format Spreadsheet"
    short_name = "Uniform Office Format Spreadsheet"
    mime = "application/vnd.uof.spreadsheet"
    extensions = [
        "uos",
    ]
    kind = Spreadsheet

    format = UniformOfficeFormatText
    signatures = []
    name = "Uniform Office Format Text"
    short_name = "Uniform Office Format Text"
    mime = "application/vnd.uof.text"
    extensions = [
        "uot",
    ]
    kind = Document

    format = Universal3d
    signatures = [
        signature = b"U3D\0"
    ]
    name = "Universal 3D"
    short_name = "Universal 3D"
    mime = "model/u3d"
    extensions = [
        "u3d",
    ]
    kind = Model

    format = UniversalSceneDescriptionAscii
    signatures = [
        signature = b"#usda 1.0"
    ]
    name = "Universal Scene Description ASCII"
    short_name = "Universal Scene Description ASCII"
    mime = "model/x-usd"
    extensions = [
        "usda",
    ]
    kind = Model

    format = UniversalSceneDescriptionBinary
    signatures = [
        signature = b"PXR-USDC"
    ]
    name = "Universal Scene Description Binary"
    short_name = "Universal Scene Description Binary"
    mime = "model/x-usd"
    extensions = [
        "usdc",
    ]
    kind = Model

    format = UniversalSceneDescriptionZip
    signatures = []
    name = "Universal Scene Description ZIP"
    short_name = "Universal Scene Description ZIP"
    mime = "model/vnd.usdz+zip"
    extensions = [
        "usdz",
    ]
    kind = Model

    format = UniversalSubtitleFormat
    signatures = [
        signature = b"\xEF\xBB\xBF<USFSubtitles"
        signature = b"<USFSubtitles"
    ]
    name = "Universal Subtitle Format"
    short_name = "Universal Subtitle Format"
    mime = "application/x-usf"
    extensions = [
        "usf",
    ]
    kind = Subtitle

    format = UnixArchiver
    signatures = [
        signature = b"!<arch>"
    ]
    name = "UNIX archiver"
    short_name = "UNIX archiver"
    mime = "application/x-archive"
    extensions = [
        "a",
    ]
    kind = Archive

    format = UnixCompress
    signatures = [
        signature = b"\x1F\x9D"
        signature = b"\x1F\xA0"
    ]
    name = "UNIX compress"
    short_name = "UNIX compress"
    mime = "application/x-compress"
    extensions = [
        "Z",
    ]
    kind = Compressed

    format = Vcalendar
    signatures = [
        signature = b"BEGIN:VCALENDAR\r\nVERSION:1.0"
        signature = b"BEGIN:VCALENDAR\nVERSION:1.0"
        signature = b"BEGIN:VCALENDAR\rVERSION:1.0"
    ]
    name = "vCalendar"
    short_name = "vCalendar"
    mime = "text/calendar"
    extensions = [
        "vcs",
    ]
    kind = Other

    format = Vcard
    signatures = [
        signature = b"BEGIN:VCARD"
    ]
    name = "vCard"
    short_name = "vCard"
    mime = "text/vcard"
    extensions = [
        "vcf",
    ]
    kind = Other

    format = VirtualMachineDisk
    signatures = [
        signature = b"COWD\x01"
        signature = b"KDMV\x01"
        signature = b"KDMV\x02"
        signature = b"KDMV\x03"
    ]
    name = "Virtual Machine Disk"
    short_name = "Virtual Machine Disk"
    mime = "application/x-vmdk"
    extensions = [
        "vmdk",
    ]
    kind = Disk

    format = VirtualRealityModelingLanguage
    signatures = [
        signature = b"#VRML V1.0 ascii"
        signature = b"#VRML V2.0 utf8"
    ]
    name = "Virtual Reality Modeling Language"
    short_name = "Virtual Reality Modeling Language"
    mime = "model/vrml"
    extensions = [
        "wrl",
    ]
    kind = Model

    format = VirtualboxVirtualDiskImage
    signatures = [
        signature = b"<<< Oracle VM VirtualBox Disk Image >>>"
    ]
    name = "VirtualBox Virtual Disk Image"
    short_name = "VirtualBox Virtual Disk Image"
    mime = "application/x-virtualbox-vdi"
    extensions = [
        "vdi",
    ]
    kind = Disk

    format = WaveformAudio
    signatures = [
        signature = b"RIFF", b"WAVE" offset = 8
    ]
    name = "Waveform Audio"
    short_name = "Waveform Audio"
    mime = "audio/vnd.wave"
    extensions = [
        "wav",
    ]
    kind = Audio

    format = Wavpack
    signatures = [
        signature = b"wvpk"
    ]
    name = "WavPack"
    short_name = "WavPack"
    mime = "audio/wavpack"
    extensions = [
        "wv",
    ]
    kind = Audio

    format = WebApplicationArchive
    signatures = []
    name = "Web Application Archive"
    short_name = "Web Application Archive"
    mime = "application/java-archive"
    extensions = [
        "war",
    ]
    kind = Package

    format = WebOpenFontFormat
    signatures = [
        signature = b"wOFF"
    ]
    name = "Web Open Font Format"
    short_name = "Web Open Font Format"
    mime = "font/woff"
    extensions = [
        "woff",
    ]
    kind = Font

    format = WebOpenFontFormat2
    signatures = [
        signature = b"wOF2"
    ]
    name = "Web Open Font Format 2"
    short_name = "Web Open Font Format 2"
    mime = "font/woff2"
    extensions = [
        "woff2",
    ]
    kind = Font

    format = WebVideoTextTracks
    signatures = [
        signature = b"\xEF\xBB\xBFWEBVTT"
        signature = b"WEBVTT"
    ]
    name = "Web Video Text Tracks"
    short_name = "Web Video Text Tracks"
    mime = "text/vtt"
    extensions = [
        "vtt",
    ]
    kind = Subtitle

    format = WebassemblyBinary
    signatures = [
        signature = b"\0asm"
    ]
    name = "WebAssembly Binary"
    short_name = "WebAssembly Binary"
    mime = "application/wasm"
    extensions = [
        "wasm",
    ]
    kind = Executable

    format = WebassemblyText
    signatures = [
        signature = b"(module"
    ]
    name = "WebAssembly Text"
    short_name = "WebAssembly Text"
    mime = "text/wasm"
    extensions = [
        "wat",
    ]
    kind = Other

    format = Webm
    signatures = []
    name = "WebM"
    short_name = "WebM"
    mime = "video/webm"
    extensions = [
        "webm",
    ]
    kind = Video

    format = Webp
    signatures = [
        signature = b"RIFF", b"WEBP" offset = 8
    ]
    name = "WebP"
    short_name = "WebP"
    mime = "image/webp"
    extensions = [
        "webp",
    ]
    kind = Image

    format = WindowsAnimatedCursor
    signatures = [
        signature = b"RIFF", b"ACON" offset = 8
    ]
    name = "Windows Animated Cursor"
    short_name = "Windows Animated Cursor"
    mime = "application/x-navi-animation"
    extensions = [
        "ani",
    ]
    kind = Image

    format = WindowsAppBundle
    signatures = []
    name = "Windows App Bundle"
    short_name = "Windows App Bundle"
    mime = "application/vnd.ms-appx.bundle"
    extensions = [
        "appxbundle",
    ]
    kind = Package

    format = WindowsAppPackage
    signatures = []
    name = "Windows App Package"
    short_name = "Windows App Package"
    mime = "application/vnd.ms-appx"
    extensions = [
        "appx",
    ]
    kind = Package

    format = WindowsBitmap
    signatures = [
        signature = b"BM", b"\0\0" offset = 12, b"\0\0\0" offset = 15, b"\x01\x00" offset = 26
    ]
    name = "Windows Bitmap"
    short_name = "Windows Bitmap"
    mime = "image/bmp"
    extensions = [
        "bmp",
    ]
    kind = Image

    format = WindowsCursor
    signatures = [
        signature = b"\x00\x00\x02\x00"
    ]
    name = "Windows Cursor"
    short_name = "Windows Cursor"
    mime = "image/x-win-bitmap"
    extensions = [
        "cur",
    ]
    kind = Image

    format = WindowsIcon
    signatures = [
        signature = b"\x00\x00\x01\x00"
    ]
    name = "Windows Icon"
    short_name = "Windows Icon"
    mime = "image/vnd.microsoft.icon"
    extensions = [
        "ico",
    ]
    kind = Image

    format = WindowsImagingFormat
    signatures = [
        signature = b"MSWIM\0\0\0"
        signature = b"WLPWM\0\0\0"
    ]
    name = "Windows Imaging Format"
    short_name = "Windows Imaging Format"
    mime = "application/x-ms-wim"
    extensions = [
        "wim",
    ]
    kind = Archive

    format = WindowsMediaAudio
    signatures = []
    name = "Windows Media Audio"
    short_name = "Windows Media Audio"
    mime = "audio/x-ms-wma"
    extensions = [
        "wma",
    ]
    kind = Audio

    format = WindowsMediaPlaylist
    signatures = [
        signature = b"<?wpl version=\"1.0\"?>"
    ]
    name = "Windows Media Playlist"
    short_name = "Windows Media Playlist"
    mime = "application/vnd.ms-wpl"
    extensions = [
        "wpl",
    ]
    kind = Playlist

    format = WindowsMediaVideo
    signatures = []
    name = "Windows Media Video"
    short_name = "Windows Media Video"
    mime = "video/x-ms-wmv"
    extensions = [
        "wmv",
    ]
    kind = Video

    format = WindowsMetafile
    signatures = [
        signature = b"\x01\x00\x09\x00"
        signature = b"\x02\x00\x09\x00"
        signature = b"\xD7\xCD\xC6\x9A"
    ]
    name = "Windows Metafile"
    short_name = "Windows Metafile"
    mime = "image/wmf"
    extensions = [
        "wmf",
    ]
    kind = Image

    format = WindowsRecordedTvShow
    signatures = [
        signature = b"\xB7\xD8\x00\x20\x37\x49\xDA\x11\xA6\x4E\x00\x07\xE9\x5E\xAD\x8D"
    ]
    name = "Windows Recorded TV Show"
    short_name = "Windows Recorded TV Show"
    mime = "video/x-wtv"
    extensions = [
        "wtv",
    ]
    kind = Video

    format = WindowsShortcut
    signatures = [
        signature = b"\x4C\x00\x00\x00\x01\x14\x02\x00\x00\x00\x00\x00\xC0\x00\x00\x00\x00\x00\x00\x46"
    ]
    name = "Windows Shortcut"
    short_name = "Windows Shortcut"
    mime = "application/x-ms-shortcut"
    extensions = [
        "lnk",
    ]
    kind = Metadata

    format = WordperfectDocument
    signatures = [
        signature = b"\xFFWPC"
    ]
    name = "WordPerfect Document"
    short_name = "WordPerfect Document"
    mime = "application/vnd.wordperfect"
    extensions = [
        "wpd",
    ]
    kind = Document

    format = WordperfectGraphics
    signatures = [
        signature = b"\xFFWPC\x10\0\0\0\x01\x16\x01\0", b"\0\0" offset = 14
    ]
    name = "WordPerfect Graphics"
    short_name = "WordPerfect Graphics"
    mime = "application/vnd.wordperfect"
    extensions = [
        "wpg",
    ]
    kind = Image

    format = WordperfectMacro
    signatures = [
        signature = b"\xFFWPC", b"\x01\x01" offset = 8
    ]
    name = "WordPerfect Macro"
    short_name = "WordPerfect Macro"
    mime = "application/vnd.wordperfect"
    extensions = [
        "wpm",
    ]
    kind = Other

    format = WordperfectPresentations
    signatures = [
        signature = b"\xFFWPC", b"\x0A" offset = 9
    ]
    name = "WordPerfect Presentations"
    short_name = "WordPerfect Presentations"
    mime = "application/vnd.wordperfect"
    extensions = [
        "shw",
    ]
    kind = Presentation

    format = XPixmap
    signatures = [
        signature = b"/* XPM */"
    ]
    name = "X PixMap"
    short_name = "X PixMap"
    mime = "image/x-xpixmap"
    extensions = [
        "xpm",
    ]
    kind = Image

    format = Xap
    signatures = []
    name = "XAP"
    short_name = "XAP"
    mime = "application/x-silverlight-app"
    extensions = [
        "xap",
    ]
    kind = Package

    format = Xbox360Executable
    signatures = [
        signature = b"XEX1"
        signature = b"XEX2"
    ]
    name = "Xbox 360 Executable"
    short_name = "Xbox 360 Executable"
    mime = "application/x-xbox360-executable"
    extensions = [
        "xex",
    ]
    kind = Executable

    format = XboxExecutable
    signatures = [
        signature = b"XBEH"
    ]
    name = "Xbox Executable"
    short_name = "Xbox Executable"
    mime = "application/x-xbox-executable"
    extensions = [
        "xbe",
    ]
    kind = Executable

    format = XmlLocalizationInterchangeFileFormat
    signatures = [
        signature = b"\xEF\xBB\xBF<xliff"
        signature = b"<xliff"
    ]
    name = "XML Localization Interchange File Format"
    short_name = "XML Localization Interchange File Format"
    mime = "application/xliff+xml"
    extensions = [
        "xlf",
    ]
    kind = Other

    format = XmlShareablePlaylistFormat
    signatures = [
        signature = b"\xEF\xBB\xBF<playlist"
        signature = b"<playlist"
    ]
    name = "XML Shareable Playlist Format"
    short_name = "XML Shareable Playlist Format"
    mime = "application/xspf+xml"
    extensions = [
        "xspf",
    ]
    kind = Playlist

    format = Xpinstall
    signatures = []
    name = "XPInstall"
    short_name = "XPInstall"
    mime = "application/x-xpinstall"
    extensions = [
        "xpi",
    ]
    kind = Package

    format = Xz
    signatures = [
        signature = b"\xFD7zXZ\0"
    ]
    name = "XZ"
    short_name = "XZ"
    mime = "application/x-xz"
    extensions = [
        "xz",
    ]
    kind = Compressed

    format = Zip
    signatures = [
        signature = b"PK\x03\x04"
        signature = b"PK\x05\x06"
        signature = b"PK\x30\x30"
    ]
    name = "ZIP"
    short_name = "ZIP"
    mime = "application/zip"
    extensions = [
        "zip",
    ]
    kind = Archive

    format = Zoo
    signatures = [
        signature = b"ZOO"
    ]
    name = "zoo"
    short_name = "zoo"
    mime = "application/x-zoo"
    extensions = [
        "zoo",
    ]
    kind = Archive

    format = Zpaq
    signatures = [
        signature = b"7kSt"
        signature = b"zPQ"
    ]
    name = "ZPAQ"
    short_name = "ZPAQ"
    mime = "application/x-zpaq"
    extensions = [
        "zpaq",
    ]
    kind = Archive

    format = Zstandard
    signatures = [
        signature = b"\x28\xB5\x2F\xFD"
    ]
    name = "Zstandard"
    short_name = "Zstandard"
    mime = "application/zstd"
    extensions = [
        "zst",
    ]
    kind = Compressed

    format = Unknown
    signatures = []
    name = "Unknown"
    short_name = "Unknown"
    mime = "application/x-unknown"
    extensions = [
        "bin",
    ]
    kind = Other
}

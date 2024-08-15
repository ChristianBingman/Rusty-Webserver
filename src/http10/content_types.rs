#[derive(Debug)]
pub struct InvalidContentTypeErr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MimeType {
    pub content_type: ContentType,
    pub content_subtype: Vec<ContentSubtype>,
}

pub fn get_mime(value: String) -> &'static str {
    match value.as_str() {
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "gz" => "application/gzip",
        "gif" => "image/gif",
        "ico" => "image/vnd.microsoft.icon",
        "jpg" | "jpeg" => "image/jpeg",
        "js" => "text/javascript",
        "json" => "applicaton/json",
        "png" => "image/png",
        "pdf" => "applicaton/pdf",
        "txt" => "text/plain",
        "xml" => "applicaton/xhtml+xml",
        _ => "application/octet-stream"
    }
}

impl From<String> for MimeType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "html" | "htm" => MimeType { content_type: ContentType::Text, content_subtype: vec![ContentSubtype::HTML] },
            "css" => MimeType { content_type: ContentType::Text, content_subtype: vec![ContentSubtype::CSS] },
            "gz" => MimeType { content_type: ContentType::Application, content_subtype: vec![ContentSubtype::GZIP] },
            "gif" => MimeType { content_type: ContentType::Image, content_subtype: vec![ContentSubtype::GIF] },
            "ico" => MimeType { content_type: ContentType::Image, content_subtype: vec![ContentSubtype::ICO] },
            "jpg" | "jpeg" => MimeType { content_type: ContentType::Image, content_subtype: vec![ContentSubtype::JPEG] },
            "js" => MimeType { content_type: ContentType::Text, content_subtype: vec![ContentSubtype::JAVASCRIPT] },
            "json" => MimeType { content_type: ContentType::Application, content_subtype: vec![ContentSubtype::JSON] },
            "png" => MimeType { content_type: ContentType::Image, content_subtype: vec![ContentSubtype::PNG] },
            "pdf" => MimeType { content_type: ContentType::Application, content_subtype: vec![ContentSubtype::PDF] },
            "txt" => MimeType { content_type: ContentType::Text, content_subtype: vec![ContentSubtype::PLAIN] },
            "xml" => MimeType { content_type: ContentType::Application, content_subtype: vec![ContentSubtype::XML] },
            "xhtml" => MimeType { content_type: ContentType::Application, content_subtype: vec![ContentSubtype::XHTML, ContentSubtype::XML] },
            _ => MimeType { content_type: ContentType::Application, content_subtype: vec![ContentSubtype::OCTETSTREAM] }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ContentType {
    Application,
    Audio,
    Example,
    Font,
    Image,
    Model,
    Text,
    Video,
    Multipart,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ContentSubtype {
    HTML,
    XML,
    XHTML,
    OCTETSTREAM,
    CSS,
    GZIP,
    GIF,
    ICO,
    JPEG,
    JAVASCRIPT,
    JSON,
    PNG,
    PDF,
    PLAIN,
}

impl std::fmt::Display for ContentSubtype {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ContentSubtype::HTML => f.write_str("html"),
            ContentSubtype::XML => f.write_str("xml"),
            ContentSubtype::XHTML => f.write_str("xhtml+xml"),
            ContentSubtype::OCTETSTREAM => f.write_str("octet-stream"),
            ContentSubtype::CSS => f.write_str("css"),
            ContentSubtype::GZIP => f.write_str("gzip"),
            ContentSubtype::GIF => f.write_str("gif"),
            ContentSubtype::JPEG => f.write_str("jpeg"),
            ContentSubtype::JAVASCRIPT => f.write_str("javascript"),
            ContentSubtype::JSON => f.write_str("json"),
            ContentSubtype::PNG => f.write_str("png"),
            ContentSubtype::PDF => f.write_str("pdf"),
            ContentSubtype::PLAIN => f.write_str("plain"),
            ContentSubtype::ICO => f.write_str("vnd.microsoft.icon"),
        }
    }
}

impl TryFrom<&str> for ContentType {
    type Error = InvalidContentTypeErr;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "application" => Ok(Self::Application),
            "audio" => Ok(Self::Audio),
            "example" => Ok(Self::Example),
            "font" => Ok(Self::Font),
            "image" => Ok(Self::Image),
            "model" => Ok(Self::Model),
            "text" => Ok(Self::Text),
            "video" => Ok(Self::Video),
            "multipart" => Ok(Self::Multipart),
            _ => Err(InvalidContentTypeErr)
        }
    }
}

impl std::fmt::Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ContentType::Application => f.write_str("application"),
            ContentType::Audio => f.write_str("audio"),
            ContentType::Example => f.write_str("example"),
            ContentType::Font => f.write_str("font"),
            ContentType::Image => f.write_str("image"),
            ContentType::Model => f.write_str("model"),
            ContentType::Text => f.write_str("text"),
            ContentType::Video => f.write_str("video"),
            ContentType::Multipart => f.write_str("multipart"),
        }
    }
}

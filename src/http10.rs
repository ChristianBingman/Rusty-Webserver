pub mod content_codings;
pub mod content_types;
pub mod headers;
pub mod methods;
pub mod request;
pub mod response;
pub mod result_codes;

fn convert_iso_8859_1_to_utf8(lines: &Vec<u8>) -> String {
    lines.iter().map(|&c| c as char).collect()
}

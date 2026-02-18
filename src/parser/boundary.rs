use crate::error::ParseError;

const MULTIPART_FORM_DATA: &str = "multipart/form-data";
const MAX_BOUNDARY_LEN: usize = 70;

/// Extracts and validates the `boundary` parameter from a `Content-Type` value.
pub fn extract_multipart_boundary(content_type: &str) -> Result<String, ParseError> {
    let mime = content_type
        .parse::<mime::Mime>()
        .map_err(|_| ParseError::new("invalid Content-Type header"))?;

    if mime.essence_str() != MULTIPART_FORM_DATA {
        return Err(ParseError::new("Content-Type must be multipart/form-data"));
    }

    let boundary = mime
        .get_param("boundary")
        .map(|value| value.as_str())
        .ok_or_else(|| ParseError::new("missing multipart boundary parameter"))?;

    validate_boundary(boundary)?;
    Ok(boundary.to_owned())
}

fn validate_boundary(boundary: &str) -> Result<(), ParseError> {
    if boundary.is_empty() {
        return Err(ParseError::new("multipart boundary cannot be empty"));
    }

    if boundary.len() > MAX_BOUNDARY_LEN {
        return Err(ParseError::new("multipart boundary cannot exceed 70 characters"));
    }

    if boundary.ends_with(' ') {
        return Err(ParseError::new(
            "multipart boundary cannot end with whitespace",
        ));
    }

    if !boundary.chars().all(is_boundary_char) {
        return Err(ParseError::new(
            "multipart boundary contains invalid characters",
        ));
    }

    Ok(())
}

fn is_boundary_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '\'' | '(' | ')' | '+' | '_' | ',' | '-' | '.' | '/' | ':' | '=' | '?' | ' ')
}

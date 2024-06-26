use std::{io::{BufRead, BufReader, Error}, path::Path, str::FromStr};
use mime::Mime;
use mime_sniffer::MimeTypeSniffer;

pub fn guess_mime_type(
    filename: &str, reader: Box<dyn std::io::Read + Send + Sync>,
) -> Result<(Mime, Box<dyn std::io::Read + Send + Sync>), Error> {
    // Try to guess the MIME type by the file extension.
    let mime = mime_guess::from_path(Path::new(filename)).first();
    if let Some(mime) = mime {
        return Ok((mime, reader));
    }

    // Try to guess the MIME type by the file contents. Use a buffered reader to avoid consuming the
    // reader in a way that a consumer later breaks.
    let mut reader = BufReader::with_capacity(512, reader);
    if let Err(e) = reader.fill_buf() {
        return Err(e);
    }
    let buf = reader.buffer();
    let mime = buf.sniff_mime_type();

    // Check if a MIME type was found.
    let mime = Mime::from_str(match mime {
        Some(mime) => mime,
        None => "application/octet-stream",
    }).unwrap();

    // Return the MIME type and the reader rewound.
    Ok((mime, Box::new(reader)))
}

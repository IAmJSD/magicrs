use image::RgbImage;
use ocrs::{ImageSource, OcrEngine, OcrEngineParams};
use once_cell::sync::Lazy;
use rten::Model;
use std::io::Read;

// Defines the engine to use for OCR.
static ENGINE: Lazy<OcrEngine> = Lazy::new(|| {
    // Inflate the text detection model from the gzip archive bundled into the app.
    const BUNDLED_DETECT_MODEL: &[u8] =
        include_bytes!("../../build/download-models/dist/text-detection.rten.gz");
    let mut text_detection = Vec::new();
    flate2::read::GzDecoder::new(BUNDLED_DETECT_MODEL)
        .read_to_end(&mut text_detection)
        .unwrap();

    // Inflate the text recognition model from the gzip archive bundled into the app.
    const BUNDLED_REC_MODEL: &[u8] =
        include_bytes!("../../build/download-models/dist/text-recognition.rten.gz");
    let mut text_recognition = Vec::new();
    flate2::read::GzDecoder::new(BUNDLED_REC_MODEL)
        .read_to_end(&mut text_recognition)
        .unwrap();

    // Create the model.
    let detection_model = Model::load(text_detection).unwrap();
    let recognition_model = Model::load(text_recognition).unwrap();
    OcrEngine::new(OcrEngineParams {
        detection_model: Some(detection_model),
        recognition_model: Some(recognition_model),
        ..Default::default()
    })
    .unwrap()
});

// Handle figuring out the value in the image specified.
pub fn scan_text(image: RgbImage) -> String {
    // Return a blank string if we are in debug mode.
    if cfg!(debug_assertions) {
        return String::new();
    }

    // Look for text within the image.
    let input = ImageSource::from_bytes(image.as_raw(), image.dimensions()).unwrap();
    ENGINE
        .get_text(&ENGINE.prepare_input(input).unwrap())
        .unwrap()
}

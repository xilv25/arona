use image::imageops::FilterType;
use image::io::Reader as ImageReader;
use image::{Rgba, RgbaImage};
use lazy_static::lazy_static;
use log::{error, info, warn};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Mutex;

type Cache = Mutex<HashMap<String, RgbaImage>>;
lazy_static! {
    static ref CACHE: Cache = Mutex::new(HashMap::new());
}

pub async fn get_image_from_url(url: &str, width: u32, height: u32) -> RgbaImage {
    if let Some(img) = check_cache(url) {
        info!("Cache Hit for {}", url);
        return img;
    }

    info!("Downloading {}", url);
    match reqwest::get(url).await {
        Ok(resp) => match resp.bytes().await {
            Ok(bytes) => match ImageReader::new(Cursor::new(bytes)).with_guessed_format() {
                Ok(img_reader) => match img_reader.decode() {
                    Ok(dynamic_img) => {
                        info!("Successfully decoded image from {}", url);
                        let img = image::imageops::resize(
                            &dynamic_img,
                            width,
                            height,
                            FilterType::Nearest,
                        );

                        add_to_cache(url, &img);
                        img
                    }
                    Err(err) => {
                        warn!("Decoding Error: {}", err);
                        generate_default_img(width, height)
                    }
                },
                Err(err) => {
                    error!("Unexpected IO Error Occurred: {}", err);
                    // We can recover here, but maybe it's worth panicking here?
                    generate_default_img(width, height)
                }
            },

            Err(err) => {
                warn!("Response Parse failed: {:}?", err);
                generate_default_img(width, height)
            }
        },
        Err(err) => {
            warn!("Download failed: {:?}", err);
            generate_default_img(width, height)
        }
    }
}

fn check_cache(url: &str) -> Option<RgbaImage> {
    if let Ok(lock) = CACHE.lock() {
        lock.get(url).cloned()
    } else {
        None
    }
}

fn add_to_cache(url: &str, img: &RgbaImage) {
    if let Ok(ref mut lock) = CACHE.lock() {
        lock.insert(url.to_string(), img.clone());
    }
}

fn generate_default_img(width: u32, height: u32) -> RgbaImage {
    let mut img = RgbaImage::new(width, height);
    for x in 0..height {
        for y in 0..width {
            img.put_pixel(x, y, Rgba([160, 32, 240, 1]));
        }
    }

    img
}

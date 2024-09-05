use image::{imageops::FilterType, DynamicImage, GenericImageView};

pub fn image_to_ascii(img: &DynamicImage, width: u32) -> String {
    let (img_width, img_height) = img.dimensions();
    let aspect_ratio = img_height as f32 / img_width as f32;
    let char_aspect_ratio = 2.0; // 假设字符的宽高比为 2:1
    let new_height = (width as f32 * aspect_ratio / char_aspect_ratio) as u32;
    let img = img.resize_exact(width, new_height, FilterType::Nearest);
    let img = img.grayscale();
    
    let ascii_chars = ["@", "#", "S", "%", "?", "*", "+", ";", ":", ",", "."];
    let mut ascii_art = String::new();

    for y in 0..new_height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y).0[0];
            let index = (pixel as f32 / 255.0 * (ascii_chars.len() - 1) as f32) as usize;
            ascii_art.push_str(ascii_chars[index]);
        }
        ascii_art.push('\n');
    }

    ascii_art
}
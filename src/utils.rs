use std::fs;

use image::{io::Reader, RgbImage};

pub const DELIMITER: &str = "#####";

pub fn open_secret(path: &std::path::PathBuf) -> String {
    match fs::read_to_string(path) {
        Ok(secret) => secret,
        Err(error) => panic!("Error opening the secret to encode: {}", error),
    }
}

pub fn open_image(path: &std::path::PathBuf) -> RgbImage {
    match Reader::open(path) {
        Ok(file) => match file.decode() {
            Ok(image) => image.into_rgb8(),
            Err(error) => panic!("An error occured decoding the image: {:?}", error),
        },
        Err(error) => panic!("An error occured opening the image: {:?}", error),
    }
}

pub fn encode_image(mut image: RgbImage, secret: String, bits: i8) -> RgbImage {
    if bits == 0 {
        panic!("Bits to encode must be higher than 0.")
    }

    let mut secret_bits = text_to_bits(secret + DELIMITER);

    if !is_encodable(&image, &secret_bits, bits) {
        panic!("The Secret is too large to be encoded.")
    }

    'encoding: for pixel in image.pixels_mut() {
        for color in pixel.0.iter_mut() {
            if secret_bits.is_empty() {
                break 'encoding;
            }

            let mut n_bits = bits as usize;

            if n_bits > secret_bits.chars().count() {
                n_bits = secret_bits.chars().count();
            }

            let mut new_color = to_binary(*color)[..(8 - n_bits)].to_string();

            new_color.push_str(&secret_bits[..n_bits]);

            secret_bits = secret_bits[n_bits..].to_string();

            *color = u8::from_str_radix(&new_color, 2).unwrap();
        }
    }

    image
}

pub fn decode_image(image: RgbImage, bits: i8) -> String {
    if bits == 0 {
        panic!("Bits to decode must be higher than 0.")
    }

    let mut secret = String::from("");
    let mut char = String::from("");

    'decoding: for pixel in image.pixels() {
        for color in pixel.0.iter() {
            if secret.ends_with(DELIMITER) {
                break 'decoding;
            }

            let mut n_bits = (8 - bits) as usize;

            if secret.ends_with(&DELIMITER[..(DELIMITER.len() - 1)])
                && (bits as usize) + char.len() >= 8
            {
                n_bits = char.len();
            }

            char.push_str(&to_binary(*color)[n_bits..]);

            if char.len() >= 8 {
                secret.push(char::from_u32(u32::from_str_radix(&char[..8], 2).unwrap()).unwrap());

                char = String::from(&char[8..]);
            }
        }
    }

    if !secret.ends_with(DELIMITER) {
        panic!("Use the same amount of encoding bits for decoding the Image Secret.")
    }

    secret.replace(DELIMITER, "")
}

fn to_binary(number: u8) -> String {
    format!("{:0>8}", format!("{:b}", number))
}

fn is_encodable(image: &RgbImage, secret: &str, bits: i8) -> bool {
    let chunks = ((secret.chars().count() as f64 / bits as f64).ceil()) as i64;
    let n_bytes = (image.pixels().len() * 3) as i64;

    if chunks > n_bytes {
        return false;
    }

    true
}

fn text_to_bits(text: String) -> String {
    let mut bits = String::from("");

    for byte in text.into_bytes() {
        bits += &to_binary(byte);
    }

    bits
}

#[cfg(test)]
pub mod tests {
    use std::fs::File;

    use super::*;

    use image::Rgb;
    use std::io::Write;
    use tempdir::TempDir;

    #[test]
    #[should_panic(
        expected = "An error occured opening the image: Os { code: 2, kind: NotFound, message: \"No such file or directory\" }"
    )]
    fn open_image_file() {
        let tmp_dir = TempDir::new("tmp").unwrap();
        let image_path = tmp_dir.path().join("image.png");

        open_image(&image_path);
    }

    #[test]
    #[should_panic(
        expected = "An error occured decoding the image: Unsupported(UnsupportedError { format: Unknown, kind: Format(Unknown) })"
    )]
    fn error_open_image_file() {
        let tmp_dir = TempDir::new("tmp").unwrap();
        let secret_path = tmp_dir.path().join("secret.txt");

        File::create(&secret_path).unwrap();

        open_image(&secret_path);
    }

    #[test]
    fn open_secret_file() {
        let tmp_dir = TempDir::new("tmp").unwrap();
        let secret_path = tmp_dir.path().join("secret.txt");
        let mut tmp_file = File::create(&secret_path).unwrap();

        writeln!(tmp_file, "This is a secret.").unwrap();

        assert_eq!(open_secret(&secret_path), "This is a secret.\n");
    }

    #[test]
    #[should_panic(
        expected = "Error opening the secret to encode: No such file or directory (os error 2)"
    )]
    fn error_open_secret_file() {
        let tmp_dir = TempDir::new("tmp").unwrap();
        let secret_path = tmp_dir.path().join("secret.txt");

        open_secret(&secret_path);
    }

    #[test]
    fn string_to_bits() {
        assert_eq!(text_to_bits(String::from("Hi")), "0100100001101001");
        assert_eq!(text_to_bits(String::from("30")), "0011001100110000");
        assert_eq!(text_to_bits(String::from("@#")), "0100000000100011");
    }

    #[test]
    fn number_to_binary() {
        assert_eq!(to_binary(64), "01000000");
    }

    #[test]
    #[should_panic(expected = "Bits to decode must be higher than 0.")]
    fn decode_minimum_bits_on_each_color() {
        decode_image(mock_image(), 0);
    }

    #[test]
    #[should_panic(expected = "Bits to encode must be higher than 0.")]
    fn encode_minimum_bits_on_each_color() {
        encode_image(mock_image(), text_to_bits(String::from("hi")), 0);
    }

    #[test]
    fn decode_image_secret() {
        assert_eq!(decode_image(encoded_image(), 6), String::from("hi"));
    }

    #[test]
    #[should_panic(
        expected = "Use the same amount of encoding bits for decoding the Image Secret."
    )]
    fn error_decode_image_secret() {
        assert_eq!(decode_image(encoded_image(), 7), String::from("hi"));
    }

    #[test]
    #[should_panic(expected = "The Secret is too large to be encoded.")]
    fn error_to_encode_large_secret_into_picture() {
        encode_image(mock_image(), String::from("Heyo"), 1);
    }

    #[test]
    fn encode_image_secret() {
        assert_eq!(
            encode_image(mock_image(), String::from("hi"), 6),
            encoded_image()
        );
    }

    #[test]
    fn encode_and_decode() {
        for i in 1..9 {
            assert_eq!(
                decode_image(encode_image(rand_image(), String::from("hi"), i), i),
                String::from("hi")
            );
        }
    }

    fn rand_image() -> RgbImage {
        let width: u32 = 5;
        let height: u32 = 4;

        let mut img = RgbImage::new(width, height);

        for w in 0..width {
            for h in 0..height {
                img.put_pixel(w, h, Rgb([225, 104, 175]));
            }
        }

        return img;
    }

    fn mock_image() -> RgbImage {
        let mut img = RgbImage::new(2, 3);

        img.put_pixel(0, 0, Rgb([225, 12, 99]));
        img.put_pixel(1, 0, Rgb([155, 2, 50]));

        img.put_pixel(0, 1, Rgb([99, 51, 15]));
        img.put_pixel(1, 1, Rgb([15, 55, 22]));

        img.put_pixel(0, 2, Rgb([155, 61, 87]));
        img.put_pixel(1, 2, Rgb([63, 30, 17]));

        return img;
    }

    //Image Encoded with "hi"
    fn encoded_image() -> RgbImage {
        let mut img = RgbImage::new(2, 3);

        img.put_pixel(0, 0, Rgb([218, 6, 100]));
        img.put_pixel(1, 0, Rgb([163, 8, 50]));

        img.put_pixel(0, 1, Rgb([76, 35, 8]));
        img.put_pixel(1, 1, Rgb([15, 55, 22]));

        img.put_pixel(0, 2, Rgb([155, 61, 87]));
        img.put_pixel(1, 2, Rgb([63, 30, 17]));

        return img;
    }
}

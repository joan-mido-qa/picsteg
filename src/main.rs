mod utils;

use std::fs::File;
use std::io::Write;

use clap::{Parser, Subcommand};
use image::RgbImage;
use utils::{decode_image, encode_image, open_image, open_secret};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(short, long)]
    bits: Option<i8>,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Encode {
        input_path: std::path::PathBuf,
        secret_path: std::path::PathBuf,
        output_path: std::path::PathBuf,
    },
    Decode {
        image_path: std::path::PathBuf,
        secret_path: std::path::PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    let bits: i8 = cli.bits.unwrap_or(1);

    match &cli.command {
        Command::Encode {
            input_path,
            secret_path,
            output_path,
        } => {
            if output_path.extension().unwrap() != "png" {
                panic!("Image must be saved with PNG format.");
            }

            let mut image: RgbImage = open_image(input_path);

            let secret: String = open_secret(secret_path);

            image = encode_image(image, secret, bits);

            if let Err(error) = image.save(output_path) {
                panic!("Encoded image could not be saved: {}", error)
            }
        }
        Command::Decode {
            image_path,
            secret_path,
        } => {
            let image: RgbImage = open_image(image_path);
            let secret: String = decode_image(image, bits);

            match File::create(secret_path) {
                Ok(mut file) => {
                    if let Err(error) = write!(file, "{}", secret) {
                        panic!("Secret could not be written: {error}")
                    }
                }
                Err(error) => panic!("Secret file could not be crated: {}", error),
            }
        }
    }
}

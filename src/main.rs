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
    #[clap(short, long, help="Number of bits to encode or decode per byte. Default: 1")]
    bits: Option<i8>,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[clap(about = "Encode an image with a secret")]
    Encode {
        #[clap(short, long, help = "Path to an encodable image")]
        image: std::path::PathBuf,
        #[clap(short, long, help = "Path to the secret to be encoded")]
        secret: std::path::PathBuf,
        #[clap(short, long, help = "Output path of the encoded image")]
        output: std::path::PathBuf,
    },
    #[clap(about = "Decode a secret from an image")]
    Decode {
        #[clap(short, long, help = "Path to an encoded image")]
        image: std::path::PathBuf,
        #[clap(short, long, help = "Output path of the decoded secret")]
        output: std::path::PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    let bits: i8 = cli.bits.unwrap_or(1);

    match &cli.command {
        Command::Encode {
            image,
            secret,
            output,
        } => {
            if output.extension().unwrap() != "png" {
                panic!("Image must be saved with PNG format.");
            }

            let mut image: RgbImage = open_image(image);

            let secret: String = open_secret(secret);

            image = encode_image(image, secret, bits);

            if let Err(error) = image.save(output) {
                panic!("Encoded image could not be saved: {}", error)
            }
        }
        Command::Decode {
            image,
            output,
        } => {
            let image: RgbImage = open_image(image);
            let secret: String = decode_image(image, bits);

            match File::create(output) {
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

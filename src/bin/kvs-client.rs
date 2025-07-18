use bincode::{config, decode_from_slice, encode_to_vec};
use clap::{Parser, Subcommand};
use serde::Serialize;
use std::{
    io::{Read, Write},
    net::TcpStream,
};

// Cli Parser
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long)]
    addr: String,
}

#[derive(Subcommand, Serialize)]
enum Commands {
    #[allow(non_camel_case_types)]
    /// Set a key-value pair
    set { key: String, val: String },

    /// Get the value for a key
    #[allow(non_camel_case_types)]
    get { key: String },

    /// Remove a key-value pair
    #[allow(non_camel_case_types)]
    rm { key: String },
}

fn main() {
    let cli = Cli::parse();
    let config = config::standard();

    // Return the helping description if they didnt specify any arguments
    if cli.command.is_none() {
        Cli::parse_from(["kvs", "--help"]);
        return;
    }

    // Bind to the address
    let mut stream = match TcpStream::connect(&cli.addr) {
        Ok(stream) => stream,
        Err(e) => {
            panic!("ERROR: {}", e);
        }
    };

    // Match the command
    match cli.command.unwrap() {
        Commands::set { key, val } => {
            let command = [0_u8];

            let bytekey = encode_to_vec(key, config).unwrap();
            let byteval = encode_to_vec(val, config).unwrap();

            /* println!(
                "Data sent: {:?} {:?} {:?} {:?} {:?}",
                command.clone(),
                bytekey.len(),
                byteval.len(),
                bytekey.clone(),
                byteval.clone()
            ); */

            let _ = stream.write(&command);
            let _ = stream.write(&[bytekey.len() as u8]);
            let _ = stream.write(&[byteval.len() as u8]);
            let _ = stream.write(&bytekey[..]);
            let _ = stream.write(&byteval[..]);
        }

        Commands::get { key } => {
            let command = [1_u8];

            let bytekey = encode_to_vec(key, config).unwrap();

            /* println!(
                "Data sent: {:?} {:?} {:?} {:?}",
                command.clone(),
                bytekey.len(),
                [0_u8],
                bytekey.clone(),
            ); */

            let _ = stream.write(&command);
            let _ = stream.write(&[bytekey.len() as u8]);
            let _ = stream.write(&[0_u8]);
            let _ = stream.write(&bytekey[..]);
            let _ = stream.write(&[]);

            let _ = stream.shutdown(std::net::Shutdown::Write);

            let mut size: [u8; 1] = [0];

            match stream.read_exact(&mut size) {
                Ok(_) => (),
                Err(_) => {
                    println!("Key not found");
                    return;
                }
            }

            let mut buf: Vec<u8> = Vec::new();
            match stream.read_to_end(&mut buf) {
                Ok(_) => (),
                Err(_) => {
                    println!("Key not found");
                    return;
                }
            }

            let byte: Result<(String, usize), bincode::error::DecodeError> =
                decode_from_slice(&buf[..], config::standard());

            match byte {
                Ok(b) => println!("{}", b.0),
                Err(_) => println!("Key not found"),
            }
        }

        Commands::rm { key } => {
            let command = [2_u8];

            let bytekey = encode_to_vec(key, config).unwrap();

            /* println!(
                "Data sent: {:?} {:?} {:?} {:?}",
                command.clone(),
                bytekey.len(),
                [0_u8],
                bytekey.clone(),
            ); */

            let _ = stream.write(&command);
            let _ = stream.write(&[bytekey.len() as u8]);
            let _ = stream.write(&[0_u8]);
            let _ = stream.write(&bytekey[..]);
            let _ = stream.write(&[]);
        }
    }
}

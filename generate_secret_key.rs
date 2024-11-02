use rand::{Rng, distributions::Alphanumeric};
use std::fs::File;
use std::io::prelude::*;

fn generate_random_key() -> [u8; 16] {
    let rng = rand::thread_rng();
    let key: Vec<u8> = rng.sample_iter(&Alphanumeric).take(16).collect();
    
    // Create a fixed-size array to store the key
    let mut result: [u8; 16] = [0; 16];
    result.copy_from_slice(&key);
    
    result
}

fn main() {
    let key = generate_random_key();
    
    // Write the key to the secret key file
    let mut file = File::create("./secret.key").expect("Unable to create file");
    file.write_all(&key).expect("Unable to write data");
    
    println!("Randomly generated key has been written to the secret key file.");
}
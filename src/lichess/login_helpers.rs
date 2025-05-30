use base64::engine::general_purpose;
use base64::Engine;
use rand::Rng;
use sha2::digest::generic_array::typenum::U32;
use sha2::{digest::generic_array::GenericArray, Digest, Sha256};

fn sha256(buffer: String) -> GenericArray<u8, U32> {
    let mut hasher = Sha256::new();
    hasher.update(buffer.as_bytes());
    hasher.finalize()
}

pub fn base64_encode<T: AsRef<[u8]>>(s: T) -> String {
    general_purpose::STANDARD
        .encode(s)
        .replace('+', "-")
        .replace('/', "_")
        .replace('=', "")
}

pub fn create_verifier() -> String {
    let random_bytes = rand::rng().random::<[u8; 32]>();
    base64_encode(random_bytes)
}

pub fn create_challenge(verifier: &String) -> String {
    base64_encode(sha256(String::from(verifier)))
}

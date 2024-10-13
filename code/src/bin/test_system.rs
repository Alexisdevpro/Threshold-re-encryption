
use std::fs::{self, create_dir_all, File, remove_dir_all, remove_file};
use std::io::{BufWriter, Read, Write};
use std::path::Path;
use tfhe::core_crypto::prelude::*;
use crate::lwe_functions::{ecrypt_text, generate_alea_encrypt, keygen, PublicKey};
use crate::secret_sharing::shared_vector_for_two_machines;
use crate::from_file::{append_vectors_to_files, keys_from_file};


mod secret_sharing;
mod lwe_functions;
mod from_file;
const Q: f64 = (2 << 15) as f64;
const ALICEKEY: &str = "keys/cleAlice.txt";
const BOBKEY: &str = "keys/cleBob.txt";


fn generate_system_keys_and_shares() {
    let path = Path::new("test_key");
    if path.exists() { remove_dir_all(path).expect("pb suppression"); }
    create_dir_all(path).expect("pb creation");


    let (pk, sk) = keygen();
    let (r, _, _) = generate_alea_encrypt();

    let secret_vector: Vec<i64> = sk.s.iter().map(|&x| x as i64).collect();
    let r_vector: Vec<i64> = r.iter().map(|&x| x as i64).collect();

    let (shares_machine_1, shares_machine_2) = shared_vector_for_two_machines(secret_vector, 2, Q as i64, 2);
    let (r_for_machine_1, r_for_machine_2) = shared_vector_for_two_machines(r_vector, 2, Q as i64, 10);

    append_vectors_to_files(
        vec![
            ("test_key/key_sys.txt", vec![&pk.a.data.as_vec(), &pk.b.data.as_vec(), &sk.s.data.as_vec()]),
            ("test_key/machine1.txt", vec![&shares_machine_1.data.as_vec(), &r_for_machine_1.data.as_vec()]),
            ("test_key/machine2.txt", vec![&shares_machine_2.data.as_vec(), &r_for_machine_2.data.as_vec()]),
        ]
    );
}

pub fn generate_bob_key() {
    let (pk, sk) = keygen();
    let path = Path::new("keys");
    if path.exists() { remove_file(BOBKEY).expect("pb suppression"); }

    append_vectors_to_files(
        vec![
            ("test_key/keypair_bob.txt", vec![&pk.a.data.as_vec(), &pk.b.data.as_vec(), &sk.s.data.as_vec()]),
        ]
    );
}

fn encrypt_plaintext_from_file() {
    let (public_key, _) = keys_from_file("test_key/key_sys.txt").unwrap();
    let mut plaintext_file = File::open("plaintext/plaintext.txt").unwrap();
    let mut plaintext_content = String::new();
    plaintext_file.read_to_string(&mut plaintext_content);

    if let Ok(number) = plaintext_content.trim().parse::<i64>() {
        let ciphertext = lwe_functions::encrypt(&public_key, number);
        let mut ciphertext_file = File::create("ciphertexts/ciphertexts.txt").unwrap();
        ciphertext_file.write_all(&format!("{:?}", ciphertext).into_bytes()).unwrap();
    } else {
        let text = plaintext_content.trim();
        let ciphertext = ecrypt_text(&public_key, text);
        let mut ciphertext_file = File::create("ciphertexts/ciphertexts.txt").unwrap();
        ciphertext_file.write_all(&format!("{:?}", ciphertext).into_bytes()).unwrap();
    }
}



fn main() {
    generate_system_keys_and_shares();
    generate_bob_key();
    encrypt_plaintext_from_file()
}



use nalgebra::{DMatrix, DVector};
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufWriter};
use std::path::Path;
use crate::lwe_functions::{PublicKey, SecretKey};
use std::fs::{self, create_dir_all, remove_dir_all, remove_file};
use std::io::{ Read, Write};


fn read_lines<P>(filename: P) -> io::Result<Vec<String>>
    where
        P: AsRef<Path>,
{
    let file = File::open(filename)?;
    let buf_reader = io::BufReader::new(file);
    buf_reader.lines().collect()
}

fn parse_list(line: &str) -> Vec<f64> {
    line.trim().trim_matches(|c| c == '[' || c == ']').split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect()
}

//recuperer les cle dans le fichier
pub fn keys_from_file(file_path: &str) -> io::Result<(PublicKey, SecretKey)> {
    let lines = read_lines(file_path)?;
    let list1 = parse_list(&lines[0]);
    let list2 = parse_list(&lines[1]);
    let list3 = parse_list(&lines[2]);
    let matrix_a = DMatrix::from_vec(25, 25, list1);
    let vector_b = DVector::from_vec(list2);
    let vector_s = DVector::from_vec(list3);
    let public_key = PublicKey { a: matrix_a, b: vector_b };
    let secret_key = SecretKey { s: vector_s };
    Ok((public_key, secret_key))
}

//recuperer les randomness dans le fichier
pub fn randomness_from_file(file_path: &str) -> io::Result<(DVector<f64>, f64)> {
    let lines = read_lines(file_path)?;
    let list1 = parse_list(&lines[0]);
    let list2 = parse_list(&lines[1]);
    let vector_1 = DVector::from_vec(list1);
    let scalar_2 = list2.first().unwrap().clone();
    Ok((vector_1, scalar_2))
}

//recupere les shares dans le fichier
pub fn shared_from_file(file_path: &str) -> io::Result<(DVector<f64>, DVector<f64>)> {
    let lines = read_lines(file_path)?;
    let list1 = parse_list(&lines[0]);
    let list2 = parse_list(&lines[1]);
    let vector_1 = DVector::from_vec(list1);
    let vector_2 = DVector::from_vec(list2);
    Ok((vector_1, vector_2))
}

pub fn append_vectors_to_files(file_vector_pairs: Vec<(&str, Vec<&Vec<f64>>)>) {
    for (filename, vectors) in file_vector_pairs {
        for data in vectors {
            append_to_file(filename, data);
        }
    }
}

pub fn append_to_file(filename: &str, data: &Vec<f64>) {
    let file = OpenOptions::new().append(true).create(true)
        .open(filename)
        .expect("pb ouverture ");

    let mut writer = BufWriter::new(file);
    write!(writer, "{:?}\n", data).expect("pb ecriture");
}

fn main() {
    let (public_key, secret_key) = keys_from_file("keys/cleAlice.txt").unwrap();
    let (randomness, scalar) = randomness_from_file("keys/randomness").unwrap();
    println!("public_key: {:?}", public_key);
    }
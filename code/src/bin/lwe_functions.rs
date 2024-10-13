use std::thread::sleep;
use nalgebra::{DMatrix, DVector};
use nalgebra::storage::Storage;
use rand_distr::{Distribution, Normal, Uniform};
use tfhe::core_crypto::prelude::*;

//Modulo en puissance de 2
const Q: f64 = (2 << 15) as f64;
//Dimension
const N : i32 = 25;
const q_sur_2: i64 = Q as i64/ 2i64;

//Definition des structures
#[derive(Debug, Clone)]
pub struct PublicKey {
    pub a: DMatrix<f64>,
    pub b: DVector<f64>,
}
#[derive(Debug, Clone)]
pub struct SecretKey {
    pub s: DVector<f64>,
}

#[derive(Debug, Clone)]
pub struct CyperText {
    pub c1: DVector<f64>,
    pub c2: f64,
}

//Definition de notre modulo
pub(crate) trait Modulo {
    fn modulo(self) -> f64;
}
impl Modulo for f64 {
    fn modulo(self) -> f64 {
        let mut x = self;
        while x < -Q / 2.0 {
            x += Q;
        }
        while x >= Q / 2.0 {
            x -= Q;
        }
        x
    }
}

pub fn keygen() -> (PublicKey, SecretKey) {
    let mut random_gen = rand::thread_rng();
    //Parametre de la distribution gaussienne (https://ia.cr/2009/391)
    let alpha = (2.0 * std::f64::consts::PI).sqrt() * Q.powf(-0.75);
    let sigma = (Q * alpha) / (2.0 * std::f64::consts::PI).sqrt();
    let sqrt_q_24 = (Q / 24.0).sqrt().round();

    //Generation des loi de proba
    let loi_uniforme_matrix = Uniform::from(-q_sur_2..q_sur_2);
    let loi_normal_vect_e = Normal::new(0.0, sigma).unwrap();
    let loi_normal_vect_s = Uniform::from(-1..2);

    //Vecteurs s (Cle secrete)
    let vect_s: Vec<f64> = (0..N)
        .map(|_| ((loi_normal_vect_s.sample(&mut random_gen))as f64).round())
        .collect();

    //Vecteurs E
    let vect_e: Vec<f64> = (0..N)
        .map(|_| {
            let value = loi_normal_vect_e.sample(&mut random_gen).round().abs();
            value.min(sqrt_q_24)
        })
        .collect();

    //Transforme les Vec en DVector
    let s = DVector::from_vec(vect_s);
    let e = DVector::from_vec(vect_e).transpose();

    //Generation de la matrice A
    let mut vecteur_a = Vec::new();
    for _ in 0..N {
        let row: Vec<f64> = (0..N)
            .map(|_| loi_uniforme_matrix.sample(&mut random_gen) as f64)
            .collect();
        vecteur_a.push(row);
    }
    let a = DMatrix::from_vec(N as usize, N as usize, vecteur_a.into_iter().flatten().collect());

    let b =  &s.transpose() *&a + &e;
    let b_to_vector = DVector::from_vec(b.iter().map(|x| x.modulo()).collect());

    //Creation de la cle publique et privee
    let public_key = PublicKey { a, b: b_to_vector };
    let secret_key = SecretKey { s };

    return (public_key, secret_key)
}

//Fonction de generation de vecteur aleatoire r et e pour le chiffrement
pub fn generate_alea_encrypt() -> (DVector<f64>, DVector<f64>, f64) {
    let mut random_gen = rand::thread_rng();
    //(Q/24) pour avoir un e final << Q/4 (voir Overleaf)
    let sqrt_q_24 = (Q / 24.0).sqrt().round();
    let q_24 = Q / 24.0;

    let loi_normal = Normal::<f64>::new(0.0, 2.0).unwrap();
    //Vecteur alea r
    let r: DVector<f64> = DVector::from_iterator(N as usize,
        (0..N).map(|_| {
            let value = loi_normal.sample(&mut random_gen).round().abs();
            value.min(sqrt_q_24)
        })
    );
    //Vecteur e1
    let e1: DVector<f64> = DVector::from_iterator(
        N as usize,
        (0..N).map(|_| {
            let value = loi_normal.sample(&mut random_gen).round().abs();
            value.min(q_24)
        })
    );
    //Vecteur e2
    let e2 = {
        let value = loi_normal.sample(&mut random_gen).round().abs();
        value.min(q_24)
    };
    return (r, e1, e2)
}


//Fonction de chiffrement de Regev
pub fn encrypt(pk: &PublicKey, delta_m: i64) -> CyperText {
    //Cle publique (a,b=As+E)
    let a = &pk.a;
    let b = &pk.b;
    //Generation de r, e1, e2 (aleas)
    let (r, e1, e2) = generate_alea_encrypt();
    //Generation de c1 et c2 (chiffrement) via les formules Regev (voir Overleaf)
    let c1 = ((a * &r) + &e1).map(|x| x.modulo());
    let c2 = ((&b.transpose() * &r)[0] + e2 + delta_m as f64).modulo() as f64;

    return CyperText { c1, c2 }
}

pub fn E(pk: &PublicKey, delta_m: f64, alea_r: DVector<f64>, e1_prime :DVector<f64>, e2_prine : f64) -> CyperText {
    let a = &pk.a;
    let b = &pk.b;
    let alea_r = alea_r;
    let e1 = e1_prime;
    let e2 = e2_prine;
    let c1 = ((a * &alea_r) + &e1).map(|x| x.modulo());
    let c2 = ((&b.transpose() * &alea_r)[0] + e2 + delta_m).modulo() as f64;
    return CyperText { c1, c2 }
}

//Renvoie l'entier le plus proche de x
fn entier_plus_proche(x: f64) -> i64 {
    let ceil_value = x.ceil() as i64;
    let floor_value = x.floor() as i64;
    let distance_ceil = (ceil_value as f64 - x).abs();
    let distance_floor = (floor_value as f64 - x).abs();

    let mut rounded_value = if distance_ceil < distance_floor {
        ceil_value
    } else if distance_floor < distance_ceil {
        floor_value
    } else {
        ceil_value
    };
    if rounded_value < 0 {
        rounded_value = 0;
    } else if rounded_value > 1 {
        rounded_value = 1;
    }
    return rounded_value
}

pub fn decrypt(sk: &SecretKey, cypher: &CyperText) -> f64 {
    let s = &sk.s;
    let c1 = &cypher.c1;
    let c2 = cypher.c2;

    let prod_sk_c1 = s.dot(c1);
    let delta_min1 = 2.0 / Q;
    let mut deltaM_with_noise = c2 - prod_sk_c1;
    //Modulo
    while deltaM_with_noise < 0.0 {
        deltaM_with_noise += Q;
    }
    let estimation_m = deltaM_with_noise.modulo().abs() * delta_min1 ;
    //println!("Estimation {}", estimation_m);
    let m_decrypt = entier_plus_proche(estimation_m);
    m_decrypt as f64
}


//multiplication modulo Gal(2,e)pour n =3
fn multiply_vectors(x: [i32; 2], y: [i32; 2]) -> [i32; 2] {
    match (x, y) {
        ([0, 0], _) | (_, [0, 0]) => [0, 0],
        ([1, 0], _) => y,
        (_, [1, 0]) => x,
        ([1, 1], [1, 1]) => [0, 1],
        ([0, 1], [0, 1]) => [1, 1],
        ([0, 1], [1, 1]) | ([1, 1], [0, 1]) => [1, 0],
        _ => [0, 0],
    }
}

//Test Decrypr(Encrypt(m))
fn test_encryption_decryption() -> Result<(), String>{
    let mut rng = rand::thread_rng();
    let loi_uniforme_pour_message = Uniform::from(0..2);
    let (pk, sk) = keygen();

    for _ in 0..10 {
        let m = loi_uniforme_pour_message.sample(&mut rng);
        let delta_m = (Q as i64 / 2) * m as i64;

        let cipher_test = encrypt(&pk, delta_m);
        let m_decrypted = decrypt(&sk, &cipher_test);
        println!("m decrypt et M = {} {} ", m_decrypted, m);
        assert_eq!(m, m_decrypted as i64);
    }
    Ok(())
}

pub fn ecrypt_text(pub_key: &PublicKey, message: &str ) -> Vec<CyperText> {
    println!("Message Aice {:?}", message);
    let mut cyphers = Vec::new();
    for byte in message.as_bytes() {
        for i in (0..8).rev() {
            let m = (byte >> i) & 1;
            let delta_m = (Q as i64 / 2) * m as i64;
            let cypher = encrypt(pub_key, delta_m);
            cyphers.push(cypher);
        }
    }
    cyphers

}

fn main() {
    //test_encryption_decryption();
    //test_encryption_decryption();
}

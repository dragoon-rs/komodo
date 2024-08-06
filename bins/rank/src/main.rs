use ark_bls12_381::Fr;
use ark_ff::Field;
use ark_std::rand::{Rng, RngCore};

use komodo::algebra::linalg::Matrix;

fn rand<T: Field, R: RngCore>(rng: &mut R) -> T {
    let element: u128 = rng.gen();
    T::from(element)
}

fn main() {
    let mut rng = rand::thread_rng();

    let elements = std::env::args()
        .skip(1)
        .map(|r| {
            r.clone()
                .split(',')
                .map(|x| {
                    if x == "-1" {
                        rand(&mut rng)
                    } else {
                        let y: u128 = x.parse().unwrap();
                        Fr::from(y)
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let m = Matrix::from_vec_vec(elements).unwrap();

    println!("{}", m.rank());
}

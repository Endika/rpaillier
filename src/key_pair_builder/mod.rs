use num::bigint::{ BigInt, RandBigInt, Sign };
use num::traits::{ Zero, One };
use num::integer::Integer;
use rand::{ OsRng, StdRng };

mod key_pair;
pub use self::key_pair::KeyPair;

use public_key::PublicKey;
use private_key::PrivateKey;

use bigint_extensions::{ Two, Three, ModPow, ModInverse };

pub struct KeyPairBuilder {
    bits: usize,
    certainty: u32
}

impl KeyPairBuilder {
    pub fn new() -> KeyPairBuilder {
        KeyPairBuilder { bits: 512, certainty: 4 }
    }

    pub fn bits(&mut self, bits: usize) -> &mut KeyPairBuilder {
        self.bits = bits;
        self
    }

    pub fn certainty(&mut self, certainty: u32) -> &mut KeyPairBuilder {
        self.certainty = certainty;
        self
    }

    pub fn finalize(&self) -> KeyPair {
        let mut sec_rng = match OsRng::new() {
            Ok(g) => g,
            Err(e) => panic!("Failed to obtain OS RNG: {}", e)
        };

        println!("generate p and q ...");

        let p = &generate_possible_prime(&mut sec_rng, self.bits, self.certainty);
        let q = &generate_possible_prime(&mut sec_rng, self.bits, self.certainty);

        println!("done!");

        let n = p * q;
        let n_squared = &n * &n;

        // recommended bit size for p and q
        // L= 1024, N= 160
        // L= 2048, N= 224
        // L= 2048, N= 256
        // L= 3072, N= 256
        let p_minus_one = p - BigInt::one();
        let q_minus_one = q - BigInt::one();

        println!("generate lambda");

        let lambda = Integer::lcm(&p_minus_one, &q_minus_one);

        println!("generate g");
        // let mut g = BigInt::two();
        // let mut helper = calculate_l(&g.mod_pow(&lambda, &n_squared), &n);;

        let mut g;
        let mut helper;
        loop {
        // while {
            g = BigInt::from_biguint(Sign::Plus, sec_rng.gen_biguint(self.bits));
            helper = calculate_l(&g.mod_pow(&lambda, &n_squared), &n);

            let a = helper.gcd(&n);
            if a == BigInt::one() {
                break;
            }
        }

        println!("done!");

        let public_key =
            PublicKey {
                bits: self.bits,
                n: n.clone(),
                n_squared: n_squared,
                g: g.clone()
            };

        println!("create private key");
        let private_key = PrivateKey {
                lambda: lambda,
                denominator: helper.mod_inverse(&n).unwrap()
            };

        KeyPair { public_key: public_key, private_key: private_key }
    }

}

fn calculate_l(u: &BigInt, n: &BigInt) -> BigInt{
    let r = u - BigInt::one();
    r / n
}


fn generate_possible_prime(sec_rng: &mut OsRng, bits: usize, certainty: u32) -> BigInt {
    let mut pp;
    loop {
        pp = BigInt::from_biguint(Sign::Plus, sec_rng.gen_biguint(bits));
        if pp.is_even() {
            continue;
        }
        if miller_rabin(&pp, certainty) {
            break;
        }
    }
    return pp;
}

fn miller_rabin(n: &BigInt, k: u32) -> bool{
    if n <= &BigInt::three() {
        return true;
    }

    let n_minus_one = n - BigInt::one();

    let mut r = 0;
    let mut s = n - BigInt::one();

    while s.clone() % BigInt::two() == BigInt::zero() {
       r += 1;
       s = s / BigInt::two();
    }

    let mut rng = match StdRng::new() {
        Ok(g) => g,
        Err(e) => panic!("Failed to obtain OS RNG: {}", e)
    };

   'outer:
   for _ in 0..k {
       let a = rng.gen_bigint_range(&BigInt::two(), &n_minus_one);
       let mut x = a.mod_pow(&s, &n);

       if x == BigInt::one() || x == n_minus_one {
            continue;
        }
        // }

       for _ in 0..(r - 1) {
           x = x.mod_pow(&BigInt::two(), &n);

           if x == BigInt::one(){
               return false;
           } else if x == n_minus_one{
               continue 'outer
            }
        }

        return false;
    }

    true
}
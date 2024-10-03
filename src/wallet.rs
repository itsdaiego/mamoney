use crate::key_pair::KeyPair;
use ring::{digest, pbkdf2};
use sha2::{Digest, Sha256};
use std::fs;

pub struct Wallet {
    pub address: String,
    pub key_pairs: Vec<KeyPair>,
}

impl Wallet {
    pub fn generate_mnemonic_words(sequence_in_bits: Vec<char>) -> Vec<String> {
        let mut hasher = Sha256::new();

        let sequence_string: String = sequence_in_bits
            .iter()
            .map(|x| -> String { x.to_string() })
            .collect();

        hasher.update(sequence_string);

        let checksum_hash: String = format!("{:X}", hasher.finalize());

        let checksum_in_binary: String = checksum_hash
            .clone()
            .into_bytes()
            .iter()
            .map(|x| -> String { format!("0{:b}", x) })
            .collect();

        let checksum_in_binary_clone = checksum_in_binary.clone();

        let checksum_4_bit_vec: Vec<char> = checksum_in_binary_clone[0..4].chars().collect();

        let entropy_sequence = [sequence_in_bits, checksum_4_bit_vec].concat();

        let mut sequence_segments = Vec::new();
        let mut segment_bit = String::new();

        for i in 0..entropy_sequence.len() {
            let sequence_vec_item = entropy_sequence[i].clone();
            segment_bit.push_str(&sequence_vec_item.to_string());

            if i % 11 == 0 {
                let clone_bit = segment_bit.clone();
                sequence_segments.push(clone_bit);
                segment_bit.clear();
            }
        }

        let sequence_segments_ids: Vec<i32> = sequence_segments
            .iter()
            .map(|x| -> i32 { i32::from_str_radix(x, 2).unwrap() })
            .collect();

        let mut wordlist: Vec<String> = fs::read_to_string("files/bip39wordlist.txt")
            .unwrap()
            .split("\n")
            .map(|x| -> String { x.to_string() })
            .collect();

        return sequence_segments_ids
            .iter()
            .map(|&x| -> String { wordlist.remove(x as usize) })
            .collect();
    }

    pub fn generate_seed(mnemonic_words: Vec<String>, password: &'static str) -> String {
        let mnemonic_words_string: String = mnemonic_words
            .iter()
            .map(|x| -> String { x.to_string() })
            .collect();

        let mut seed_bytes = [0u8; digest::SHA256_OUTPUT_LEN];
        let salt = format!("mnemonic{}", password);

        let pbkdf2_iterations = std::num::NonZeroU32::new(2048).unwrap();

        static PBKDF2_ALG: pbkdf2::Algorithm = pbkdf2::PBKDF2_HMAC_SHA256;

        pbkdf2::derive(
            PBKDF2_ALG,
            pbkdf2_iterations,
            salt.as_bytes(),
            mnemonic_words_string.as_bytes(),
            &mut seed_bytes,
        );

        let seed: String = seed_bytes
            .iter()
            .map(|x| -> String { format!("{:x?}", x) })
            .collect();

        return seed;
    }

    pub fn new(seed: String) -> Wallet {
        let mut public_key_hasher = Sha256::new();

        let private_key: String = seed[seed.len() / 2..].to_owned();
        public_key_hasher.update(&private_key);

        let public_key = format!("{:X}", public_key_hasher.finalize());
        let chain_code = seed[..seed.len() / 2].to_owned();

        let mut address_hasher = Sha256::new();

        address_hasher.update(&public_key);

        let address = format!("{:X}", address_hasher.finalize());

        let key_pair = KeyPair {
            private_key: private_key.clone(),
            public_key,
            chain_code,
            index: 0,
            path: "m/44'/0'/0'/0/0".to_owned(), // m: master key / 44': BIP44 / 0': coin type (Bitcoin) / 0': account / 0: change / 0: address index
        };

        let second_key_pair = KeyPair::derive_child(&key_pair, key_pair.index + 1);

        let mut key_pairs = Vec::new();
        key_pairs.push(key_pair);
        key_pairs.push(second_key_pair);

        return Wallet { key_pairs, address };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mnemonic_words() {
        let mut mnemonic_words: Vec<String> = Vec::new();
        mnemonic_words.push("bla".to_owned());

        let input: Vec<char> = [
            '0', '0', '0', '1', '1', '0', '0', '0', '0', '0', '1', '0', '1', '0', '0', '1', '0',
            '0', '1', '0', '0', '1', '1', '0', '1', '1', '1', '0', '0', '0', '0', '1', '0', '0',
            '0', '1', '0', '0', '1', '1', '0', '0', '1', '0', '1', '1', '0', '1', '0', '1', '1',
            '0', '0', '1', '1', '1', '1', '0', '1', '0', '1', '0', '1', '0', '0', '0', '0', '0',
            '0', '1', '0', '0', '1', '1', '0', '0', '0', '1', '1', '0', '0', '0', '1', '0', '1',
            '0', '1', '0', '1', '1', '1', '0', '1', '1', '1', '0', '1', '1', '0', '1', '0', '0',
            '1', '1', '1', '0', '0', '0', '0', '0', '0', '0', '1', '0', '1', '0', '0', '0', '0',
            '1', '0', '1', '1', '1', '1', '0', '1', '0',
        ]
        .iter()
        .cloned()
        .collect();

        let mnemonic_words: Vec<String> = [
            "abandon".to_owned(),
            "cost".to_owned(),
            "napkin".to_owned(),
            "illegal".to_owned(),
            "essay".to_owned(),
            "punch".to_owned(),
            "priority".to_owned(),
            "charge".to_owned(),
            "merit".to_owned(),
            "tenant".to_owned(),
            "december".to_owned(),
            "face".to_owned(),
        ]
        .iter()
        .cloned()
        .collect();

        assert_eq!(Wallet::generate_mnemonic_words(input), mnemonic_words);
    }

    #[test]
    fn test_generate_seed() {
        let mnemonic_words: Vec<String> = [
            "abandon".to_owned(),
            "cost".to_owned(),
            "napkin".to_owned(),
            "illegal".to_owned(),
            "essay".to_owned(),
            "punch".to_owned(),
            "priority".to_owned(),
            "charge".to_owned(),
            "merit".to_owned(),
            "tenant".to_owned(),
            "december".to_owned(),
            "face".to_owned(),
        ]
        .iter()
        .cloned()
        .collect();

        assert_eq!(
            Wallet::generate_seed(mnemonic_words, "superpassword"),
            "ce313b6a66b6f56fbe7a6bb8d7c84014f3fe7f36f3e768f659ead704a6c"
        );
    }

    #[test]
    fn test_new() {
        let seed = "ce313b6a66b6f56fbe7a6bb8d7c84014f3fe7f36f3e768f659ead704a6c";

        let wallet = Wallet::new(seed.to_owned());

        assert_eq!(
            wallet.address,
            "03027CC470BB03D5EBC760B58B242ACD62C188657D0C4199B9704B31AA10471E"
        );
        assert_eq!(
            wallet.key_pairs[0].private_key,
            "014f3fe7f36f3e768f659ead704a6c"
        );
        assert_eq!(
            wallet.key_pairs[0].public_key,
            "1314311DC62A41D39FA733A3E9B3D6C1A2B720D78C6C4BF527556105F746A395"
        );
        assert_eq!(
            wallet.key_pairs[0].chain_code,
            "ce313b6a66b6f56fbe7a6bb8d7c84"
        );
        assert_eq!(wallet.key_pairs[0].index, 0);
        assert_eq!(wallet.key_pairs[0].path, "m/44'/0'/0'/0/0");
    }
}

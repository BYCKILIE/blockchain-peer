use std::fs;
use openssl::encrypt::{Decrypter, Encrypter};
use openssl::pkey::{PKey, Public};
use openssl::rsa::Rsa;
use tokio::io;

pub struct RsaEncryption {
    public_pkey: PKey<Public>,
    private_pkey: PKey<openssl::pkey::Private>,
}

impl RsaEncryption {
    pub fn new(path: &str) -> RsaEncryption {
        let public_pem = fs::read(format!("{}/public.pem", path)).expect("Failed to read public key");
        let public_key = Rsa::public_key_from_pem(&public_pem).expect("Failed to load public key");
        let public_pkey = PKey::from_rsa(public_key).expect("Failed to create public PKey");

        let private_pem = fs::read(format!("{}/private.pem", path)).expect("Failed to read private key");
        let private_key = Rsa::private_key_from_pem(&private_pem).expect("Failed to load private key");
        let private_pkey = PKey::from_rsa(private_key).expect("Failed to create private PKey");

        RsaEncryption {
            public_pkey,
            private_pkey,
        }
    }

    pub fn parse_public(pem_key: &[u8]) -> io::Result<PKey<Public>> {
        let public_key = Rsa::public_key_from_pem(pem_key)?;
        let public_pkey = PKey::from_rsa(public_key)?;

        Ok(public_pkey)
    }

    pub fn encrypt(message: &[u8], pkey: &PKey<Public>) -> Vec<u8> {
        let mut encrypter = Encrypter::new(pkey).expect("Failed to create encrypter");
        encrypter.set_rsa_padding(openssl::rsa::Padding::PKCS1).unwrap();

        let mut encrypted = vec![0; encrypter.encrypt_len(message).unwrap()];
        let encrypted_len = encrypter.encrypt(message, &mut encrypted).expect("Encryption failed");
        encrypted.truncate(encrypted_len);

        encrypted
    }

    pub fn decrypt(&self, encrypted: &[u8]) -> Vec<u8> {
        let mut decrypter = Decrypter::new(&self.private_pkey).expect("Failed to create decrypter");
        decrypter.set_rsa_padding(openssl::rsa::Padding::PKCS1).unwrap();

        let mut decrypted = vec![0; decrypter.decrypt_len(encrypted).unwrap()];
        let decrypted_len = decrypter.decrypt(encrypted, &mut decrypted).expect("Decryption failed");
        decrypted.truncate(decrypted_len);

        decrypted
    }

    pub fn get_public_pkey(&self) -> Vec<u8> {
        self.public_pkey.public_key_to_pem().unwrap()
    }

    pub fn clone(&self) -> RsaEncryption {
        RsaEncryption {
            public_pkey: PKey::from_rsa(self.public_pkey.rsa().unwrap().clone()).unwrap(),
            private_pkey: PKey::from_rsa(self.private_pkey.rsa().unwrap().clone()).unwrap(),
        }
    }
}
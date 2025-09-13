use openssl::symm::{Cipher, Crypter, Mode};
use rand::Rng;

pub struct AesEncryption {
    key: [u8; 32],
    iv: [u8; 16],
}

impl AesEncryption {
    pub fn new() -> Self {
        let mut rng = rand::rng();

        let key: [u8; 32] = rng.random();
        let iv: [u8; 16] = rng.random();

        AesEncryption { key, iv }
    }
    
    pub fn get_key(&self) -> &[u8; 32] {
        &self.key
    }
    
    pub fn get_iv(&self) -> &[u8; 16] {
        &self.iv
    }
    
    pub fn encrypt(message: &[u8], key: &[u8; 32], iv: &[u8; 16]) -> Vec<u8> {
        let cipher = Cipher::aes_256_cbc();
        let mut encrypter = Crypter::new(cipher, Mode::Encrypt, key, Some(iv)).unwrap();
        let mut ciphertext = vec![0; message.len() + cipher.block_size()];
        let mut count = encrypter.update(message, &mut ciphertext).unwrap();
        count += encrypter.finalize(&mut ciphertext[count..]).unwrap();
        ciphertext.truncate(count);
        ciphertext
    }
    
    pub fn decrypt(&self, ciphertext: &[u8]) -> Vec<u8> {
        let cipher = Cipher::aes_256_cbc();
        let mut decrypter = Crypter::new(cipher, Mode::Decrypt, &self.key, Some(&self.iv)).unwrap();
        let mut decrypted = vec![0; ciphertext.len() + cipher.block_size()];
        let mut count = decrypter.update(&ciphertext, &mut decrypted).unwrap();
        count += decrypter.finalize(&mut decrypted[count..]).unwrap();
        decrypted.truncate(count);
        decrypted
    }

    pub fn clone(&self) -> AesEncryption {
        AesEncryption {
            key: self.key.clone(),
            iv: self.iv.clone(),
        }
    }
    
}

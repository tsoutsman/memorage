#[derive(Clone, Debug)]
#[allow(missing_copy_implementations)]
pub struct KeyPair([u8; 64]);

impl KeyPair {
    pub fn generate<T>(rng: &mut T) -> Self
    where
        T: rand::CryptoRng + rand::RngCore,
    {
        Self(ed25519_dalek::Keypair::generate(rng).to_bytes())
    }

    pub fn from_entropy() -> Self {
        // God help us all with the fifteen different fucking rand versions please it's 1 AM why
        // are my local builds working but CI builds failing.
        Self::generate(&mut rand::thread_rng())
    }

    pub fn public(&self) -> &[u8] {
        &self.0[..32]
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn to_public(&self) -> [u8; 32] {
        <[u8; 32]>::try_from(&self.0[..32]).unwrap()
    }

    fn private(&self) -> &[u8] {
        &self.0[32..]
    }

    pub fn to_bytes(&self) -> [u8; 64] {
        self.0
    }

    pub fn to_pkcs8(&self) -> [u8; 85] {
        // Adapted from https://github.com/briansmith/ring/blob/main/src/pkcs8.rs
        const BEFORE_PRIVATE_KEY: [u8; 16] = [
            0x30, 0x43, 0x02, 0x01, 0x01, 0x30, 0x04, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22,
            0x04, 0x20,
        ];
        const AFTER_PRIVATE_KEY: [u8; 5] = [0xa1, 0x23, 0x03, 0x21, 0x00];

        let mut bytes = [0; 85];
        bytes[..16].copy_from_slice(&BEFORE_PRIVATE_KEY);
        bytes[16..(16 + 32)].copy_from_slice(self.private());
        bytes[(16 + 32)..(16 + 32 + 5)].copy_from_slice(&AFTER_PRIVATE_KEY);
        bytes[(16 + 32 + 5)..].copy_from_slice(self.public());
        bytes
    }
}

impl AsRef<[u8]> for KeyPair {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

use crate::{PublicKey, Signature};

#[derive(Debug)]
pub struct KeyPair {
    bytes: [u8; 85],
    keypair: ring::signature::Ed25519KeyPair,
}

// TODO custom error types?

impl KeyPair {
    #[allow(clippy::missing_panics_doc)]
    #[allow(clippy::result_unit_err)]
    pub fn generate(rng: &dyn ring::rand::SecureRandom) -> Result<Self, ()> {
        use ring::signature::Ed25519KeyPair;

        let document = Ed25519KeyPair::generate_pkcs8(rng).map_err(|_| ())?;
        let keypair = Ed25519KeyPair::from_pkcs8(document.as_ref()).map_err(|_| ())?;
        // SAFETY: the length of a document is always 85 bytes.
        let bytes = document.as_ref().try_into().unwrap();

        Ok(Self { bytes, keypair })
    }

    // TODO proper errors
    pub fn from_bytes<B>(bytes: B) -> Result<Self, &'static str>
    where
        B: AsRef<[u8]>,
    {
        let bytes: [u8; 85] = bytes
            .as_ref()
            .try_into()
            .map_err(|_| "incorrect bytes length")?;
        let keypair =
            ring::signature::Ed25519KeyPair::from_pkcs8(&bytes).map_err(|_| "invalid key")?;
        Ok(Self { bytes, keypair })
    }

    pub fn sign<B>(&self, bytes: B) -> Signature
    where
        B: AsRef<[u8]>,
    {
        self.keypair.sign(bytes.as_ref()).into()
    }

    pub fn public_key(&self) -> PublicKey {
        let ring_public_key = ring::signature::KeyPair::public_key(&self.keypair);
        // :)
        unsafe { std::mem::transmute(*ring_public_key) }
    }
}

impl AsRef<[u8]> for KeyPair {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl serde::ser::Serialize for KeyPair {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let bytes = self.bytes.as_ref();
        serde_bytes::Bytes::new(bytes).serialize(serializer)
    }
}

impl<'de> serde::de::Deserialize<'de> for KeyPair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = <serde_bytes::ByteBuf>::deserialize(deserializer)?;
        Self::from_bytes(bytes).map_err(serde::de::Error::custom)
    }
}

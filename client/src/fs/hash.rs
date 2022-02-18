pub fn hash<T>(mut reader: T) -> crate::Result<[u8; 32]>
where
    T: std::io::Read,
{
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0; 65536];

    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                hasher.update(&buffer[..n]);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e.into()),
        }
    }

    Ok(hasher.finalize().into())
}

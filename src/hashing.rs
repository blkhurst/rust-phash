use crate::{errors::HashError, types as T};
use img_hash::{HashAlg as ImgAlg, Hasher, HasherConfig, ImageHash};
use std::{fs, io::Read, path::Path};

/// Stream the file and return its BLAKE3 digest.
pub fn compute_blake3(path: &Path) -> Result<String, HashError> {
    let mut f = fs::File::open(path)?;

    // Stream in 1 MB chunks
    let mut hasher = blake3::Hasher::new();
    let mut buf = [0u8; 1024 * 1024];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}

/// Map types::HashAlg to img_hash::HashAlg
fn map_alg(alg: T::HashAlg) -> ImgAlg {
    match alg {
        T::HashAlg::Mean => ImgAlg::Mean,
        T::HashAlg::Gradient => ImgAlg::Gradient,
        T::HashAlg::DoubleGradient => ImgAlg::DoubleGradient,
    }
}

/// Re-usable Hasher
pub fn build_hasher(alg: T::HashAlg, w: u32, h: u32) -> Hasher {
    HasherConfig::new()
        .hash_size(w, h)
        .hash_alg(map_alg(alg))
        .to_hasher()
}

/// Perceptual Hash Image
pub fn perceptual_hash(path: &Path, hasher: &Hasher) -> Result<String, HashError> {
    let img = img_hash::image::open(path)?; // ImageError -> HashError
    Ok(hasher.hash_image(&img).to_base64())
}

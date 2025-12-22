use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub fn uuids_from_seed(seed: &str, count: usize) -> Vec<Uuid> {
    let mut hasher = Sha256::new();
    hasher.update(seed.as_bytes());
    let seed_bytes = hasher.finalize();

    let mut rng = ChaCha20Rng::from_seed(seed_bytes.into());

    (0..count)
        .map(|_| {
            let high = rng.next_u64() as u128;
            let low = rng.next_u64() as u128;
            Uuid::from_u128((high << 64) | low)
        })
        .collect()
}

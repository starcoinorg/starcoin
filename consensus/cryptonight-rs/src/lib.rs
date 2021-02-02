use libc::{c_char, c_int, c_void, size_t};

#[link(name = "cryptonight", kind = "static")]
extern "C" {
    fn cryptonight_hash(
        data: *const c_char,
        hash: *const c_char,
        length: size_t,
        variant: c_int,
        height: u64,
    ) -> c_void;
}

const VARIANT: i32 = 4;
const HEIGHT: u64 = 0;

#[allow(clippy::unsound_collection_transmute)]
pub fn cryptonight_r(data: &[u8], size: usize) -> Vec<u8> {
    let hash: Vec<i8> = vec![0i8; 32];
    let data_ptr: *const c_char = data.as_ptr() as *const c_char;
    let hash_ptr: *const c_char = hash.as_ptr() as *const c_char;
    let mut hash = unsafe {
        cryptonight_hash(data_ptr, hash_ptr, size, VARIANT, HEIGHT);
        std::mem::transmute::<Vec<i8>, Vec<u8>>(hash)
    };
    hash.reverse();
    hash
}

#[cfg(test)]
mod tests;

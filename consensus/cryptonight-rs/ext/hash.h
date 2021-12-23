#pragma once

#include <stddef.h>
#include <iostream>

namespace crypto {

  extern "C" {
#include "hash-ops.h"
  }

#pragma pack(push, 1)
  class hash {
    char data[HASH_SIZE];
  };
  class hash8 {
    char data[8];
  };
#pragma pack(pop)

  static_assert(sizeof(hash) == HASH_SIZE, "Invalid structure size");
  static_assert(sizeof(hash8) == 8, "Invalid structure size");

  /*
    Cryptonight hash functions
  */

  inline void cn_fast_hash(const void *data, std::size_t length, hash &hash) {
    cn_fast_hash(data, length, reinterpret_cast<char *>(&hash));
  }

  inline hash cn_fast_hash(const void *data, std::size_t length) {
    hash h;
    cn_fast_hash(data, length, reinterpret_cast<char *>(&h));
    return h;
  }

  inline void cn_slow_hash(const void *data, std::size_t length, hash &hash, int variant = 0, uint64_t height = 0) {
    cn_slow_hash(data, length, reinterpret_cast<char *>(&hash), variant, 0/*prehashed*/, height);
  }

  inline void cn_slow_hash_prehashed(const void *data, std::size_t length, hash &hash, int variant = 0, uint64_t height = 0) {
    cn_slow_hash(data, length, reinterpret_cast<char *>(&hash), variant, 1/*prehashed*/, height);
  }

  inline void tree_hash(const hash *hashes, std::size_t count, hash &root_hash) {
    tree_hash(reinterpret_cast<const char (*)[HASH_SIZE]>(hashes), count, reinterpret_cast<char *>(&root_hash));
  }

  inline std::ostream &operator <<(std::ostream &o, const crypto::hash &v) {
    epee::to_hex::formatted(o, epee::as_byte_span(v)); return o;
  }
  inline std::ostream &operator <<(std::ostream &o, const crypto::hash8 &v) {
    epee::to_hex::formatted(o, epee::as_byte_span(v)); return o;
  }

  constexpr static crypto::hash null_hash = {};
  constexpr static crypto::hash8 null_hash8 = {};
}

CRYPTO_MAKE_HASHABLE(hash)
CRYPTO_MAKE_COMPARABLE(hash8)

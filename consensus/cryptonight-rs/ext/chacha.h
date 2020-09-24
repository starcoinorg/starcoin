// Copyright (c) 2014-2018, The Monero Project

#pragma once

#include <stdint.h>
#include <stddef.h>

#define CHACHA_KEY_SIZE 32
#define CHACHA_IV_SIZE 8

#if defined(__cplusplus)
#include <memory.h>

#include "memwipe.h"
#include "mlocker.h"
#include "hash.h"

namespace crypto {
  extern "C" {
#endif
    void chacha8(const void* data, size_t length, const uint8_t* key, const uint8_t* iv, char* cipher);
    void chacha20(const void* data, size_t length, const uint8_t* key, const uint8_t* iv, char* cipher);
#if defined(__cplusplus)
  }

  using chacha_key = epee::mlocked<tools::scrubbed_arr<uint8_t, CHACHA_KEY_SIZE>>;

#pragma pack(push, 1)
  // MS VC 2012 doesn't interpret `class chacha_iv` as POD in spite of [9.0.10], so it is a struct
  struct chacha_iv {
    uint8_t data[CHACHA_IV_SIZE];
  };
#pragma pack(pop)

  static_assert(sizeof(chacha_key) == CHACHA_KEY_SIZE && sizeof(chacha_iv) == CHACHA_IV_SIZE, "Invalid structure size");

  inline void chacha8(const void* data, std::size_t length, const chacha_key& key, const chacha_iv& iv, char* cipher) {
    chacha8(data, length, key.data(), reinterpret_cast<const uint8_t*>(&iv), cipher);
  }

  inline void chacha20(const void* data, std::size_t length, const chacha_key& key, const chacha_iv& iv, char* cipher) {
    chacha20(data, length, key.data(), reinterpret_cast<const uint8_t*>(&iv), cipher);
  }

  inline void generate_chacha_key(const void *data, size_t size, chacha_key& key, uint64_t kdf_rounds) {
    static_assert(sizeof(chacha_key) <= sizeof(hash), "Size of hash must be at least that of chacha_key");
    epee::mlocked<tools::scrubbed_arr<char, HASH_SIZE>> pwd_hash;
    crypto::cn_slow_hash(data, size, pwd_hash.data(), 0/*variant*/, 0/*prehashed*/, 0/*height*/);
    for (uint64_t n = 1; n < kdf_rounds; ++n)
      crypto::cn_slow_hash(pwd_hash.data(), pwd_hash.size(), pwd_hash.data(), 0/*variant*/, 0/*prehashed*/, 0/*height*/);
    memcpy(&unwrap(unwrap(key)), pwd_hash.data(), sizeof(key));
  }

  inline void generate_chacha_key_prehashed(const void *data, size_t size, chacha_key& key, uint64_t kdf_rounds) {
    static_assert(sizeof(chacha_key) <= sizeof(hash), "Size of hash must be at least that of chacha_key");
    epee::mlocked<tools::scrubbed_arr<char, HASH_SIZE>> pwd_hash;
    crypto::cn_slow_hash(data, size, pwd_hash.data(), 0/*variant*/, 1/*prehashed*/, 0/*height*/);
    for (uint64_t n = 1; n < kdf_rounds; ++n)
      crypto::cn_slow_hash(pwd_hash.data(), pwd_hash.size(), pwd_hash.data(), 0/*variant*/, 0/*prehashed*/, 0/*height*/);
    memcpy(&unwrap(unwrap(key)), pwd_hash.data(), sizeof(key));
  }

  inline void generate_chacha_key(std::string password, chacha_key& key, uint64_t kdf_rounds) {
    return generate_chacha_key(password.data(), password.size(), key, kdf_rounds);
  }
}

#endif

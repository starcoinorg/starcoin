// keccak.h
// 19-Nov-11  Markku-Juhani O. Saarinen <mjos@iki.fi>

// Copyright (c) 2014 The Boolberry developers
// Distributed under the MIT/X11 software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.


#pragma once

#include <stdint.h>
#include <string.h>
#include "hash.h"

#ifndef KECCAK_ROUNDS
#define KECCAK_ROUNDS 24
#endif

#ifndef ROTL64
#define ROTL64(x, y) (((x) << (y)) | ((x) >> (64 - (y))))
#endif

// compute a keccak hash (md) of given byte length from "in"

#define KK_MIXIN_SIZE 24

namespace crypto
{
  template<typename pod_operand_a, typename pod_operand_b>
  pod_operand_a xor_pod(const pod_operand_a& a, const pod_operand_b& b)
  {
    static_assert(sizeof(pod_operand_a) == sizeof(pod_operand_b), "invalid xor_h usage: different sizes");
    static_assert(sizeof(pod_operand_a)%8 == 0, "invalid xor_h usage: wrong size");

    hash r;
    for(size_t i = 0; i != 4; i++)
    {
      ((uint64_t*)&r)[i] = ((const uint64_t*)&a)[i] ^ ((const uint64_t*)&b)[i];
    }
    return r;
  }

#define XOR_2(A, B) crypto::xor_pod(A, B)
#define XOR_3(A, B, C) crypto::xor_pod(A, XOR_2(B, C))
#define XOR_4(A, B, C, D) crypto::xor_pod(A, XOR_3(B, C, D))
#define XOR_5(A, B, C, D, E) crypto::xor_pod(A, XOR_4(B, C, D, E))
#define XOR_8(A, B, C, D, F, G, H, I) crypto::xor_pod(XOR_4(A, B, C, D), XOR_4(F, G, H, I))




  typedef uint64_t state_t_m[25];
  typedef uint64_t mixin_t[KK_MIXIN_SIZE];

  //with multiplication, for tests
  template<class f_traits>
  int keccak_generic(const uint8_t *in, size_t inlen, uint8_t *md, size_t mdlen)
  {
    state_t_m st;
    uint8_t temp[144];
    size_t i, rsiz, rsizw;

    rsiz = sizeof(state_t_m) == mdlen ? HASH_DATA_AREA : 200 - 2 * mdlen;
    rsizw = rsiz / 8;

    memset(st, 0, sizeof(st));

    for ( ; inlen >= rsiz; inlen -= rsiz, in += rsiz) {
      for (i = 0; i < rsizw; i++)
        st[i] ^= ((uint64_t *) in)[i];
      f_traits::keccakf(st, KECCAK_ROUNDS);
    }


    // last block and padding
    memcpy(temp, in, inlen);
    temp[inlen++] = 1;
    memset(temp + inlen, 0, rsiz - inlen);
    temp[rsiz - 1] |= 0x80;

    for (i = 0; i < rsizw; i++)
      st[i] ^= ((uint64_t *) temp)[i];

    f_traits::keccakf(st, KECCAK_ROUNDS);

    memcpy(md, st, mdlen);

    return 0;
  }

  template<class f_traits, class callback_t>
  int wild_keccak(const uint8_t *in, size_t inlen, uint8_t *md, size_t mdlen, callback_t cb)
  {
    state_t_m st;
    uint8_t temp[144];
    uint64_t rsiz, rsizw;

    rsiz = sizeof(state_t_m) == mdlen ? HASH_DATA_AREA : 200 - 2 * mdlen;
    rsizw = rsiz / 8;
    memset(&st[0], 0, 25*sizeof(st[0]));
    

    for ( ; inlen >= rsiz; inlen -= rsiz, in += rsiz) 
    {
      for (size_t i = 0; i < rsizw; i++)
        st[i] ^= ((uint64_t *) in)[i];

      for(size_t ll = 0; ll != KECCAK_ROUNDS; ll++)
      {
        if(ll != 0)
        {//skip first round
          mixin_t mix_in;
          cb(st, mix_in);
          for (size_t k = 0; k < KK_MIXIN_SIZE; k++)
            st[k] ^= mix_in[k];
        }
        f_traits::keccakf(st, 1);
      }
    }

    // last block and padding
    memcpy(temp, in, inlen);
    temp[inlen++] = 1;
    memset(temp + inlen, 0, rsiz - inlen);
    temp[rsiz - 1] |= 0x80;

    for (size_t i = 0; i < rsizw; i++)
      st[i] ^= ((uint64_t *) temp)[i];

    for(size_t ll = 0; ll != KECCAK_ROUNDS; ll++)
    {
      if(ll != 0)
      {//skip first state with
        mixin_t mix_in;
        cb(st, mix_in);
        for (size_t k = 0; k < KK_MIXIN_SIZE; k++)
          st[k] ^= mix_in[k]; 
      }
      f_traits::keccakf(st, 1);
    }

    memcpy(md, st, mdlen);

    return 0;
  }

  template<class f_traits, class callback_t>
  int wild_keccak_dbl(const uint8_t *in, size_t inlen, uint8_t *md, size_t mdlen, callback_t cb)
  {
    //Satoshi's classic
    wild_keccak<f_traits>(in, inlen, md, mdlen, cb);
    wild_keccak<f_traits>(md, mdlen, md, mdlen, cb);
    return 0;
  }

  class regular_f
  {
  public:
    static void keccakf(uint64_t st[25], int rounds);
  };

  class mul_f
  {
  public:
    static void keccakf(uint64_t st[25], int rounds);
  };
}


#include <assert.h>
#include <stddef.h>
#include <stdint.h>

#include "c_jh.h"
#include "hash-ops.h"

void hash_extra_jh(const void *data, size_t length, char *hash) {
  int r = jh_hash(HASH_SIZE * 8, data, 8 * length, (uint8_t*)hash);
  assert(SUCCESS == r);
}

#include <stddef.h>
#include <stdint.h>

#include "hash-ops.h"
#include "c_skein.h"

void hash_extra_skein(const void *data, size_t length, char *hash) {
  int r = skein_hash(8 * HASH_SIZE, data, 8 * length, (uint8_t*)hash);
  assert(SKEIN_SUCCESS == r);
}

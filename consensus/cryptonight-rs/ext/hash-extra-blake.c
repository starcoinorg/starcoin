#include <stddef.h>
#include <stdint.h>

#include "c_blake256.h"

void hash_extra_blake(const void *data, size_t length, char *hash) {
  blake256_hash((uint8_t*)hash, data, length);
}

#include <stddef.h>
#include <stdint.h>

#include "c_groestl.h"

void hash_extra_groestl(const void *data, size_t length, char *hash) {
  groestl(data, length * 8, (uint8_t*)hash);
}

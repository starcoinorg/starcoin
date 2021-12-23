#include "cryptonight.h"
#include "hash-ops.h"

void cryptonight_hash(const char* input, char* output, uint32_t len, int variant, uint64_t height)
{
    cn_slow_hash(input, len, output, variant, 0, height);
}

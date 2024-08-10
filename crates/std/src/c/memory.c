#include "memory.h"
#include "phix.h"
#include "stddef.h"
#include "stdint.h"

void* memset(void* p, int c, size_t size) {
    char* cp = (char*)p;
    for (size_t i = 0; i < size; i++) {
        cp[i] = c;
    }

    return p;
}

int memcmp(void* p1, void* p2, size_t count) {
    char* c1 = (char*)p1;
    char* c2 = (char*)p2;
    while (count-- > 0) {
        if (*c1++ != *c2++)
            return c1[-1] - c2[-1];
    }

    return 0;
}

void* memcpy(void* dst, void* src, size_t size) {
    char* d = dst;
    char* s = src;

    while (size--)
        *(d++) = *(s++);

    return dst;
}

void* get_paddr(void* vaddr) { return phix_get_paddr(vaddr); }

uint8_t get_flags(void* vaddr) { return phix_get_flags(vaddr); }

#include "string.h"
#include "stdbool.h"
#include "stdio.h"
#include "stdlib.h"

bool is_digit(char c) { return (c >= 48 && c <= 57); }

char to_lower(unsigned char c) {
    if (c >= 65 && c <= 90)
        return c + 32;

    return c;
}

char to_upper(unsigned char c) {
    if (c >= 97 && c <= 122)
        return c - 32;

    return c;
}

int to_digit(char c) {
    if (!is_digit(c))
        return c;

    return c - 48;
}

size_t strlen(const char* str) {
    size_t len = 0;
    while (str[len]) {
        len++;
    }

    return len;
}

size_t strnlen(const char* str, size_t max) {
    size_t len = 0;
    while (str[len] && len != max) {
        len++;
    }

    return len;
}

char* strcpy(char* dest, const char* src) {
    char* res = dest;

    while (*src)
        *(dest++) = *(src++);

    *dest = 0;

    return res;
}

char* strncpy(char* dest, const char* src, size_t n) {
    char* res = dest;

    while (*src && n-- > 1)
        *(dest++) = *(src++);

    *dest = 0;

    return res;
}

int strncmp(const char* s1, const char* s2, size_t n) {
    unsigned char u1, u2;
    while (n-- > 0) {
        u1 = (unsigned char)*s1++;
        u2 = (unsigned char)*s2++;
        if (u1 != u2)
            return u1 - u2;
        if (u1 == '\0')
            break;
    }

    return 0;
}

char* strcpy_strip(char* dest, const char* src) {
    char* res = dest;

    while (*src && *src != 0x20)
        *(dest++) = *(src++);

    *dest = 0;

    return res;
}

int strnlen_terminator(const char* str, int max, char terminator) {
    for (int i = 0; i < max; i++) {
        if (str[i] == '\0' || str[i] == terminator)
            return i;
    }

    return max;
}

int istrncmp(const char* s1, const char* s2, size_t n) {
    unsigned char u1, u2;
    while (n-- > 0) {
        u1 = (unsigned char)*s1++;
        u2 = (unsigned char)*s2++;
        if (to_lower(u1) != to_lower(u2))
            return u1 - u2;
        if (u1 == '\0')
            break;
    }

    return 0;
}

char* sp = NULL;
char* strtok(char* str, const char* delimiters) {
    int i = 0;
    int len = strlen(delimiters);
    if (!str && !sp)
        return 0;

    if (str && !sp) {
        sp = str;
    }

    char* p_start = sp;
    while (1) {
        for (i = 0; i < len; i++) {
            if (*p_start == delimiters[i]) {
                p_start++;
                break;
            }
        }

        if (i == len) {
            sp = p_start;
            break;
        }
    }

    if (*sp == '\0') {
        sp = 0;
        return sp;
    }

    // Find end of substring
    while (*sp != '\0') {
        for (i = 0; i < len; i++) {
            if (*sp == delimiters[i]) {
                *sp = '\0';
                break;
            }
        }

        sp++;
        if (i < len)
            break;
    }

    return p_start;
}

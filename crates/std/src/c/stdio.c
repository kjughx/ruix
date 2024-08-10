#include "stdio.h"
#include "math.h"
#include "phix.h"
#include "string.h"
#include <stdarg.h>
#include <math.h>

int getkey() {
    int val;
    do {
        val = phix_getkey();
    } while (val == 0);

    return val;
}

void readline(char* buf, int max, bool output_while_typing) {
    int i = 0;
    for (i = 0; i < max - 1; i++) {
        char key = getkey();

        /* Carriage return, the line is finished */
        if (key == 13)
            break;
        if (output_while_typing)
            putchar(key);
        /* Backspace */
        if (key == 0x08 && i >= 1) {
            buf[i - 1] = 0x00;
            i -= 2; /* -2 to -1 after i++ */
            continue;
        }

        buf[i] = key;
    }

    buf[i] = 0x00;
}

char* itoa(int i) {
    static char text[12];
    int loc = 11;
    text[11] = 0;
    char neg = 1;
    if (i >= 0) {
        neg = 0;
        i = -i;
    }
    while (i) {
        text[--loc] = '0' - (i % 10);
        i /= 10;
    }

    if (loc == 11)
        text[--loc] = '0';

    if (neg)
        text[--loc] = '-';

    return &text[loc];
}

int atoi(const char* str) {
    if (!str)
        return 0;

    int ret = 0;
    size_t len = strlen(str) - 1;
    int c;
    if (len >= sizeof(int))
        return 0;

    while (*str) {
        if ((c = to_digit(*str)))
            ret += c * powi(10, len--);

        str++;
    }

    return ret;
}

int putchar(int c) {
    phix_putchar((char)c);
    return 0;
}

static char* decimal_to_hex(unsigned int decimal) {
    static char hex[12];
    int j = 11;

    while (decimal != 0) {
        int remainder = decimal % 16;
        if (remainder < 10)
            hex[--j] = 48 + remainder;
        else
            hex[--j] = 55 + remainder;

        decimal /= 16;
    }

    return &hex[j];
}

int printf(const char* fmt, ...) {
    va_list ap;
    int ival;
    char* sval;

    va_start(ap, fmt);
    for (const char* p = fmt; *p; p++) {
        if (*p != '%') {
            putchar(*p);
            continue;
        }
        switch (*++p) {
        case 'd': {
            ival = va_arg(ap, int);
            phix_print(itoa(ival));
        } break;
        case 'c': {
            ival = va_arg(ap, int);
            putchar(ival);
        } break;
        case 's': {
            sval = va_arg(ap, char*);
            phix_print(sval);
        } break;
        case 'p': {
            ival = va_arg(ap, int);
            phix_print("0X");
            phix_print(decimal_to_hex(ival));
        } break;
        default:
            putchar(*p);
        }
        va_end(ap);
    }

    return 0;
}

int open(const char* filename, const char* mode) {
    return phix_open(filename, mode);
}

int read(void* buf, size_t count, size_t n, int fd) {
    return phix_read(buf, count, n, fd);
}

int close(int fd) {
    return phix_close(fd);
}

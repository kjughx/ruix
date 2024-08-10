#include "math.h"

int powi(int b, int e) {
    while (e)
        b *= (e--);

    return b;
}

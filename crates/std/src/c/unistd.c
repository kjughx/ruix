#include "unistd.h"
#include "phix.h"

int exec(const char* filename) {
    phix_exec(filename);

    return 0;
}

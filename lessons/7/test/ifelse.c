#include <stdio.h>

// ARGS: 1 2 3
int main(int argc, char** argv) {
    register int x = 5;
    if (argc == 4) {
        x = 10;
    } else {
        x = 15;
    }

    printf("%d\n", x);

    return 0;
}
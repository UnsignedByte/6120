#include <stdio.h>

// ARGS: 1 2 3 4 5
int main(int argc, char **argv) {
    int idx = 0;

    while (idx < argc) {
        int v = idx * 3 + 4;
        printf("%d\n", v);

        idx += 2;
    }

    return 0;
}
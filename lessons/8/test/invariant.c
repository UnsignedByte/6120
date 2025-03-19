#include <stdio.h>

// ARGS: 1 2 3 4 5
int main(int argc, char **argv) {
    int idx = 0;

    if (argc == 0) {
        argc = 100;
    } else {
        argc += 1;
    }

    do {
        int y = 10;
        int v = argc / y;
        printf("%d\n", v);

        idx += v;
    } while (idx < argc);
 
    return 0;
}
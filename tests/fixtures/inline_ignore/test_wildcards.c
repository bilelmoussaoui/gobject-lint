#include <string.h>

void test_wildcard_all(const char *input) {
    char buf[100];

    // Test "all" wildcard - should ignore all rules
    /* goblint-ignore-next-line: all */
    strcpy(buf, input);
}

void test_wildcard_star(const char *input) {
    char buf[100];

    // Test "*" wildcard - should ignore all rules
    /* goblint-ignore-next-line: * */
    strcpy(buf, input);
}

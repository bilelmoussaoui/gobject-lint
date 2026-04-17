#include <glib.h>

typedef struct {
    int dummy;
} Context;

void context_terminate_with_error(Context *context, GError *error);

void test_terminate_with_error(void) {
    Context ctx;
    GError *error = NULL;

    g_file_get_contents("test.txt", NULL, NULL, &error);

    if (error) {
        g_prefix_error(&error, "Failed: ");
        // Functions named *_terminate_with_error typically take ownership
        context_terminate_with_error(&ctx, error);
        return;
    }
}

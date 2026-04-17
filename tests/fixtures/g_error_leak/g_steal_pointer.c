#include <glib.h>

typedef struct {
    GError *error;
} ErrorContainer;

ErrorContainer *create_error_container(void) {
    GError *error = NULL;
    ErrorContainer *container;

    g_file_get_contents("test.txt", NULL, NULL, &error);

    if (error) {
        container = g_new0(ErrorContainer, 1);
        // Transfer ownership via g_steal_pointer - not a leak
        container->error = g_steal_pointer(&error);
        return container;
    }

    return NULL;
}

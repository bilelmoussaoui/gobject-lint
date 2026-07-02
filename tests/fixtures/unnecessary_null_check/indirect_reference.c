#include <glib.h>

typedef struct {
  char *command_line;
} Node;

/* NOT redundant: the null check protects against dereferencing node */
void
test_field_dereference (const char *name)
{
  Node *node = NULL;
  if (node) {
    g_clear_pointer (&node->command_line, g_free);
  }
}

/* NOT redundant: addr is a pointer-to-pointer, g_clear_pointer dereferences it */
void
test_pointer_to_pointer (char **addr)
{
  if (addr) {
    g_clear_pointer (addr, g_free);
  }
}

/* Redundant: &var is never NULL and g_clear_pointer checks *pp */
void
test_address_of_local (void)
{
  char *ptr = NULL;
  if (ptr) {
    g_clear_pointer (&ptr, g_free);
  }
}

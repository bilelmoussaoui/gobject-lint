#include <glib.h>

static void
test_different_variable (const char *type_name, char *new_type_name, char *another_var)
{
  /* Correct: checking a different variable than the one being freed (block style) */
  if (type_name == NULL)
    g_free (new_type_name);

  /* Correct: inline if - checking different variable before freeing */
  if (type_name == NULL) g_free (new_type_name);

  /* Correct: inline if - another different variable case */
  if (another_var != NULL) g_free (new_type_name);

  /* Wrong: checking the same variable before freeing it (block style) */
  if (new_type_name != NULL)
    g_free (new_type_name);

  /* Wrong: inline if - checking same variable before freeing */
  if (another_var != NULL) g_free (another_var);
}

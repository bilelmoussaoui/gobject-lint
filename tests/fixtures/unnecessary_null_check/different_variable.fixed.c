#include <glib.h>

static void
test_different_variable (const char *type_name, char *new_type_name, char *another_var, char **deref_var, GList *list)
{
  /* Correct: checking a different variable than the one being freed (block style) */
  if (type_name == NULL)
    g_free (new_type_name);

  /* Correct: inline if - checking different variable before freeing */
  if (type_name == NULL) g_free (new_type_name);

  /* Correct: inline if - another different variable case */
  if (another_var != NULL) g_free (new_type_name);

  /* Correct: checking a variable name which is part of the name of the variable being freed (block style) */
  if (list != NULL)
    g_free (list->data);
  if (deref_var != NULL)
    g_free (*deref_var);

  /* Wrong: checking the same variable before freeing it (block style) */
  g_free (new_type_name);

  /* Wrong: inline if - checking same variable before freeing */
  g_free (another_var);

  /* Wrong: checking the same variable before freeing it with different formatting (block style) */
  g_free (list ->data);
}

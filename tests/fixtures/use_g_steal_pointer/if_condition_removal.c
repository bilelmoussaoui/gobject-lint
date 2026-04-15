#include <glib.h>

typedef struct {
  char *theme_node;
  char *value;
} PrivData;

/* Case 1: if condition should be dropped - assignment to different variable */
static void
test_if_drop_1 (PrivData *priv, char **a_value, char *charset_str)
{
  if (charset_str)
    {
      *a_value = charset_str;
      charset_str = NULL;
    }
}

/* Case 2: if condition should be dropped - assignment to local */
static void
test_if_drop_2 (PrivData *priv)
{
  char *old_theme_node;

  if (priv->theme_node)
    {
      old_theme_node = priv->theme_node;
      priv->theme_node = NULL;
    }
}

/* Case 3: if condition should be dropped - simple member assignment */
static void
test_if_drop_3 (PrivData *priv, char *new_value)
{
  if (new_value)
    {
      priv->value = new_value;
      new_value = NULL;
    }
}

/* Case 4: if condition should be KEPT - checks different variable */
static void
test_if_keep (PrivData *priv, char *new_value, gboolean some_flag)
{
  if (some_flag)
    {
      priv->value = new_value;
      new_value = NULL;
    }
}

/* Case 5: if condition should be KEPT - checks NULL on different variable */
static void
test_if_keep_null_check (PrivData *priv, char *new_value, char *some_param)
{
  if (some_param == NULL)
    {
      priv->value = new_value;
      new_value = NULL;
    }
}

/* Case 6: if/else on SAME variable should be REMOVED (g_steal_pointer handles NULL) */
static void
test_if_else_drop (PrivData *priv, char *new_value)
{
  if (new_value != NULL)
    {
      priv->value = new_value;
      new_value = NULL;
    }
  else
    {
      priv->value = NULL;
    }
}

/* Case 7: if/else on DIFFERENT variable should be KEPT */
static void
test_if_else_keep (PrivData *priv, char *new_value, char *some_param)
{
  if (some_param == NULL)
    {
      priv->value = new_value;
      new_value = NULL;
    }
  else
    {
      priv->value = NULL;
    }
}

/* Case 8: if/else with EXTRA statements in else - should be KEPT */
static void
test_if_else_keep_extra (PrivData *priv, char *new_value)
{
  if (new_value != NULL)
    {
      priv->value = new_value;
      new_value = NULL;
    }
  else
    {
      priv->value = NULL;
      priv->theme_node = NULL;
    }
}

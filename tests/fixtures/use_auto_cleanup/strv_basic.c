#include <glib.h>

static void
strv_with_strsplit (const char *input)
{
  gchar **tokens = NULL;

  tokens = g_strsplit (input, ":", -1);
  g_print ("%s\n", tokens[0]);
  g_strfreev (tokens);
}

static void
strv_with_strsplit_set (const char *input)
{
  gchar **tokens = NULL;

  tokens = g_strsplit_set (input, ":;,", -1);
  g_print ("%s\n", tokens[0]);
  g_strfreev (tokens);
}

static void
strv_with_strdupv (const char **input)
{
  gchar **copy = NULL;

  copy = g_strdupv ((gchar **) input);
  g_print ("%s\n", copy[0]);
  g_strfreev (copy);
}

static void
strv_not_freed (const char *input)
{
  gchar **tokens = NULL;

  tokens = g_strsplit (input, ":", -1);
  g_print ("%s\n", tokens[0]);
}

static gchar **
strv_returned (const char *input)
{
  gchar **tokens = NULL;

  tokens = g_strsplit (input, ":", -1);
  return tokens;
}

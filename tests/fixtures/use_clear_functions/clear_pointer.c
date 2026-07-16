#include <glib.h>

#define my_macro(arg) arg

// Windows COM object, see https://github.com/bilelmoussaoui/gobject-linter/issues/180
static void ITfSource_Release (gpointer data) {}

typedef struct
{
  gchar *str;
} MyStruct;

static void
my_free_func (MyStruct *my_struct)
{
  g_free (my_struct->str);
  g_free (my_struct);
}

static void
my_func (gchar *str, MyStruct *my_struct)
{
  // Suggest g_clear_pointer on function pointers
  g_free (str);
  str = NULL;

  if (str != NULL) {
    g_free (str);
    str = NULL;
  }

  my_free_func (my_struct);
  my_struct = NULL;

  if (my_struct != NULL) {
    my_free_func (my_struct);
    my_struct = NULL;
  }

  // Do not suggest g_clear_pointer on macros
  my_macro (my_struct);
  my_struct = NULL;

  if (my_struct != NULL) {
    my_macro (my_struct);
    my_struct = NULL;
  }

  ITfSource_Release (my_struct);
  my_struct = NULL;

  if (my_struct != NULL) {
    ITfSource_Release (my_struct);
    my_struct = NULL;
  }
}

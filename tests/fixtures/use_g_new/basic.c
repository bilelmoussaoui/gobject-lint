#include <glib.h>

typedef struct { int x; } MyStruct;

static void
my_func (void)
{
  MyStruct *s = g_malloc (sizeof (MyStruct));
  MyStruct *z = g_malloc0 (sizeof (MyStruct));
}

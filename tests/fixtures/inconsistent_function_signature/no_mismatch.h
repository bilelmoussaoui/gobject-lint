#pragma once

#include <glib.h>

gint   bar_get_count (void);
gchar *bar_get_name  (void);

/* unsigned int vs unsigned are the same C type */
unsigned int bar_get_flags (void);

/* enum Foo vs Foo are the same type */
typedef enum BarMode { MODE_A, MODE_B } BarMode;
void bar_set_mode (enum BarMode mode);

#pragma once

#include <glib.h>
#include <stdbool.h>

gint   bar_get_count (void);
gchar *bar_get_name  (void);

/* unsigned int vs unsigned are the same C type */
unsigned int bar_get_flags (void);

/* enum Foo vs Foo are the same type */
typedef enum BarMode { MODE_A, MODE_B } BarMode;
void bar_set_mode (enum BarMode mode);

/* enum return type: enum Foo vs Foo */
enum BarMode bar_get_mode (void);

/* _Bool vs bool are the same C99 type */
_Bool bar_is_active (void);

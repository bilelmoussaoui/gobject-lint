#include "no_mismatch.h"

int
bar_get_count (void)
{
  return 42;
}

char *
bar_get_name (void)
{
  return "name";
}

unsigned
bar_get_flags (void)
{
  return 0;
}

void
bar_set_mode (BarMode mode)
{
}

BarMode
bar_get_mode (void)
{
  return MODE_A;
}

bool
bar_is_active (void)
{
  return 1;
}

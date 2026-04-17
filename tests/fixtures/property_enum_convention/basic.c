#include <glib-object.h>

/* First enum: missing PROP_0, has = 0 */
typedef enum {
  PROP_NAME = 0,
  PROP_TITLE,
  PROP_DESCRIPTION
} MyObjectProps;

/* Second enum: missing PROP_0 - will conflict with first when fixed */
typedef enum {
  PROP_FOO,
  PROP_BAR
} MyObjectProps2;

/* Third enum: already correct with prefix */
typedef enum {
  WIDGET_PROPS_PROP_0,
  PROP_WIDTH,
  PROP_HEIGHT
} WidgetProps;

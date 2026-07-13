#include "basic.h"

/* _get_type with G_GNUC_CONST after params — should warn */
GType my_widget_get_type (void);

/* _get_type with G_GNUC_CONST before return type — should warn */
GType my_other_get_type (void);

/* _get_type with G_GNUC_PURE after params — should warn */
GType my_pure_get_type (void);

/* non-_get_type with G_GNUC_CONST — should NOT warn */
GType not_a_get_type_func (void) G_GNUC_CONST;

/* _get_type without any attribute — should NOT warn */
GType my_normal_get_type (void);

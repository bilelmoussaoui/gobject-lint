#include <adwaita.h>
#include <glib/gi18n.h>

void test_adwaita(void) {
    AdwToast *toast;
    AdwMessageDialog *dialog;
    AdwStatusPage *status_page;

    // Should be flagged
    toast = adw_toast_new("Operation complete");

    // Should NOT be flagged - already wrapped
    toast = adw_toast_new(_("Saved successfully"));

    // Should be flagged
    adw_toast_set_button_label(toast, "Undo");

    // Should be flagged
    dialog = ADW_MESSAGE_DIALOG(adw_message_dialog_new(NULL, "Warning", "Details"));

    // Should be flagged
    adw_message_dialog_set_body(dialog, "This action cannot be undone");

    // Should NOT be flagged
    adw_message_dialog_set_body(dialog, _("Are you sure?"));

    // Should be flagged
    adw_status_page_set_title(status_page, "No Results Found");
}

pub trait DialogReport<E> {
    fn report_dialog(self) -> Self;
    fn report_dialog_msg(self, msg: &str) -> Self;
    fn report_dialog_with<F: FnOnce(&E) -> String>(self, f: F) -> Self;
}

impl<T, E> DialogReport<E> for Result<T, E>
where
    E: std::fmt::Debug,
{
    fn report_dialog(self) -> Self {
        if let Err(ref e) = self {
            native_dialog::MessageDialog::new()
                .set_type(native_dialog::MessageType::Error)
                .set_title("Error")
                .set_text(&format!("{:?}", e))
                .show_alert()
                .expect("Error while displaying an error message box (needs KDialog on Linux)");
        }

        self
    }

    fn report_dialog_msg(self, msg: &str) -> Self {
        if self.is_err() {
            native_dialog::MessageDialog::new()
                .set_type(native_dialog::MessageType::Error)
                .set_title("Error")
                .set_text(msg)
                .show_alert()
                .expect("Error while displaying an error message box (needs KDialog on Linux)");
        }

        self
    }

    fn report_dialog_with<F: FnOnce(&E) -> String>(self, f: F) -> Self {
        if let Err(ref e) = self {
            native_dialog::MessageDialog::new()
                .set_type(native_dialog::MessageType::Error)
                .set_title("Error")
                .set_text(&f(e))
                .show_alert()
                .expect("Error while displaying an error message box (needs KDialog on Linux)");
        }

        self
    }
}

pub fn report_error(msg: &str) {
    native_dialog::MessageDialog::new()
        .set_type(native_dialog::MessageType::Error)
        .set_title("Error")
        .set_text(msg)
        .show_alert()
        .expect("Error while displaying an error message box (needs KDialog on Linux)");
}

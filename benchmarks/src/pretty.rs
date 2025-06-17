pub(crate) fn filesize(bytes: usize) -> String {
    size::Size::from_bytes(bytes)
        .format()
        .with_base(size::Base::Base10)
        .with_style(size::Style::Abbreviated)
        .to_string()
}

pub(crate) mod bar {
    use colored::Colorize;

    pub(crate) fn dimmed(
        maybe_bar: Option<&indicatif::ProgressBar>,
        maybe_msg: Option<String>,
    ) -> Option<String> {
        if let Some(bar) = maybe_bar {
            let previous = bar.message();
            let msg = if let Some(msg) = maybe_msg {
                msg
            } else {
                bar.message()
            };
            bar.set_message(msg.dimmed().to_string());
            Some(previous)
        } else {
            None
        }
    }

    pub(crate) fn normal(
        maybe_bar: Option<&indicatif::ProgressBar>,
        maybe_msg: Option<String>,
    ) -> Option<String> {
        if let Some(bar) = maybe_bar {
            let previous = bar.message();
            let msg = if let Some(msg) = maybe_msg {
                msg
            } else {
                bar.message()
            };
            bar.set_message(msg.normal().to_string());
            Some(previous)
        } else {
            None
        }
    }
}

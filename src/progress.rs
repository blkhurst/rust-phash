use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

pub fn multi() -> MultiProgress {
    MultiProgress::new()
}

pub fn bar(len: u64, msg: &str) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:20.cyan/blue}] {pos}/{len} ({eta})")
            .expect("template")
            .progress_chars("=>-"),
    );
    pb.set_message(msg.to_string());
    pb
}

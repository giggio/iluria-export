use indicatif::{ProgressBar, ProgressStyle};

pub static mut BAR: Option<ProgressBar> = None;

pub fn start_progress_bar(len: u64) {
    unsafe {
        let bar = ProgressBar::new(len);
        bar.set_style(ProgressStyle::default_bar().template("{wide_bar}"));
        BAR = Some(bar);
    }
}

pub fn set_progress_bar_len(len: u64) {
    unsafe {
        if let Some(bar) = &BAR {
            let old_position = bar.position() as f64;
            let old_len = bar.length() as f64;
            bar.set_position(0); // todo: remove this and set them pos and len together when https://github.com/mitsuhiko/indicatif/issues/236 is done
            bar.set_length(len);
            bar.set_position((old_position / old_len * (len as f64)).round() as u64);
        }
    }
}

pub fn inc_progress_bar(amount: u64) {
    unsafe {
        if let Some(bar) = &BAR {
            bar.inc(amount);
        }
    }
}

pub fn finish_progress_bar() {
    unsafe {
        if let Some(bar) = &BAR {
            bar.finish();
            BAR = None;
        }
    }
}

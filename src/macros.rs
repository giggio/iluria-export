macro_rules! printlnv {
        ($($arg:tt)*) => ({
            unsafe {
                if $crate::VERBOSE {
                    if let Some(bar) = &$crate::run::BAR {
                        bar.println(format!($($arg)*));
                    } else {
                        println!($($arg)*);
                    }
                }
            }
        })
    }

macro_rules! printlnpb {
        ($($arg:tt)*) => ({
            #[allow(unused_unsafe)] // todo: this is a bug in the compiler, remove when https://github.com/rust-lang/rust/issues/49112 is fixed.
            unsafe {
                if let Some(bar) = &$crate::run::BAR {
                    bar.println(format!($($arg)*));
                } else {
                    println!($($arg)*);
                }
            }
        })
    }

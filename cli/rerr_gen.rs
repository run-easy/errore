use std::io::Write;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(version = "1.0.0", about = "Used to generate the error.rs file")]
pub struct Arg {
    #[arg(
        short,
        long,
        help = "Module name. must be globally unique",
        value_name = "MODULE NAME"
    )]
    name: String,
    #[arg(
        short,
        long,
        help = "Module path, which must be an absolute path",
        value_name = "MODULE PATH",
        default_value_t = String::from("crate")
    )]
    path: String,
    #[arg(
        short,
        long,
        help = "Add the log component, which adds a log trace to the build object file.",
        value_name = "BOOL",
        default_value_t = false
    )]
    log: bool,
    target: String,
}

impl Arg {
    fn build(&self) {
        let mut f = match std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(self.target.as_str())
        {
            Ok(f) => f,
            Err(e) => {
                log::error!("{}", e);
                std::process::exit(1);
            }
        };

        match f.write_all(self.generate_rust().as_bytes()) {
            Err(e) => {
                log::error!("{}", e);
                std::process::exit(1);
            }
            Ok(_) => {
                log::info!("Generate {} success!", self.target);
                std::process::exit(0);
            }
        }
    }

    fn generate_rust(&self) -> String {
        let mut raw_string = String::from(
            "
mod internel {
    #![allow(dead_code)]
    use errore::error::*;
    use errore::kind::*;

    #[ctor::ctor]
    static _RMODULE: RModule = RModule::new(\"MODULE_NAME\");

    #[inline]
    pub(crate) fn new_simple(kind: RErrorKind) -> RError {
        RError::new_simple(_RMODULE.clone(), kind)
    }
    #[inline]
    pub(crate) fn new_simple_msg(kind: RErrorKind, msg: &'static str) -> RError {
        RError::new_simple_msg(_RMODULE.clone(), kind, msg)
    }
    #[inline]
    pub(crate) fn new_custom_msg(kind: RErrorKind, msg: String) -> RError {
        RError::new_custom_msg(_RMODULE.clone(), kind, msg)
    }
}

#[allow(unused_imports)]
pub use internel::*;
#[allow(unused_imports)]
pub use errore::kind::*;
#[allow(unused_imports)]
pub use errore::error::RError;

#[macro_export]
macro_rules! throw_rerr {
        ($kind:expr) => {
            return MODULE_PATH::new_simple($kind).to_err()
        };
        ($kind:expr,$msg:expr) => {
            return MODULE_PATH::new_simple_msg($kind, $msg).to_err()
        };
        ($kind:expr,$($args:tt)*) => {
            return MODULE_PATH::new_custom_msg($kind,format!($($args)*)).to_err()
        }
    }

#[macro_export]
macro_rules! ignore_rerr {
    ($err:expr) => {
        let _: usize = { &$err as *const errore::error::RError as usize };
        log::warn!(\"{}. Ignore it.\", $err);
        $err.ignore()
    };
}

#[macro_export]
macro_rules! block_rerr {
    ($err:expr) => {
        let _: usize = { &$err as *const errore::error::RError as usize };
        log::warn!(\"{}. Block it.\", $err);
        $err.ignore()
    };
}

#[macro_export]
macro_rules! new_rerr {
    ($kind:expr) => {
            MODULE_PATH::new_simple($kind)
        };
        ($kind:expr,$msg:expr) => {
            MODULE_PATH::new_simple_msg($kind, $msg)
        };
        ($kind:expr,$($args:tt)*) => {
            MODULE_PATH::new_custom_msg($kind,format!($($args)*))
        };
}
    ",
        );

        raw_string = raw_string.replace("MODULE_NAME", &self.name);
        raw_string = raw_string.replace("MODULE_PATH", &self.path);

        if !self.log {
            raw_string = raw_string.replace(
                "log::warn!(\"{}. Ignore it.\", $err);",
                "// log::warn!(\"{}. Ignore it.\", $err);",
            );

            raw_string = raw_string.replace(
                "log::warn!(\"{}. Block it.\", $err);",
                "// log::warn!(\"{}. Block it.\", $err);",
            );
        }

        raw_string
    }
}

fn main() {
    std::env::set_var("RUST_LOG", "debug");
    pretty_env_logger::init();
    let arg = Arg::parse();
    arg.build();
}

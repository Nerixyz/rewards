use std::fmt::{Debug, Display};

pub trait ResultExt {
    fn log_if_err(&self, op: &str);
    fn dbg_if_err(&self, op: &str);
}

impl<T, E> ResultExt for Result<T, E>
where
    E: Display + Debug,
{
    fn log_if_err(&self, op: &str) {
        if let Err(e) = self {
            log::warn!("{op}: {e}");
        }
    }

    fn dbg_if_err(&self, op: &str) {
        if let Err(e) = self {
            log::warn!("{op}: {e:?}");
        }
    }
}

pub trait ResultCExt<T, E2> {
    fn err_into(self) -> Result<T, E2>;
}

impl<T, E1, E2> ResultCExt<T, E2> for Result<T, E1>
where
    E1: Into<E2>,
{
    fn err_into(self) -> Result<T, E2> {
        self.map_err(Into::into)
    }
}

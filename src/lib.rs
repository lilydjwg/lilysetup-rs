use std::io::IsTerminal;

use tracing_subscriber::EnvFilter;
pub use eyre::{Result, WrapErr, bail};
pub use tracing::{trace, debug, info, warn, error};

pub fn setup_logging(default_level: &str) -> Result<()> {
  let filter = EnvFilter::try_from_default_env()
    .unwrap_or_else(|_| EnvFilter::from(default_level));
  let isatty = std::io::stderr().is_terminal();
  let is_connected_to_journald = is_connected_to_journald();
  let fmt = tracing_subscriber::fmt::fmt()
    .with_writer(std::io::stderr)
    .with_env_filter(filter)
    .with_ansi(isatty);
  if !is_connected_to_journald {
    fmt
      .with_timer(tracing_subscriber::fmt::time::ChronoLocal::new(
          String::from("%Y-%m-%d %H:%M:%S%.6f %z")))
      .init();
  } else {
    fmt.without_time().init();
  }

  Ok(())
}

fn is_connected_to_journald() -> bool {
  is_connected_to_journald_impl().is_some()
}

// https://github.com/systemd/systemd/blob/main/docs/JOURNAL_NATIVE_PROTOCOL.md
fn is_connected_to_journald_impl() -> Option<()> {
  let s = std::env::var("JOURNAL_STREAM").ok()?;
  let (dev, inode) = s.split_once(':')?;
  let dev = dev.parse().ok()?;
  let inode = inode.parse().ok()?;
  let stat = stat_stderr()?;
  if stat.st_dev == dev && stat.st_ino == inode {
    Some(())
  } else {
    None
  }
}

fn stat_stderr() -> Option<libc::stat> {
  use std::mem::MaybeUninit;

  let mut buf = MaybeUninit::uninit();
  let ret = unsafe { libc::fstat(2, buf.as_mut_ptr()) };
  if ret == 0 {
    Some(unsafe { buf.assume_init() })
  } else {
    None
  }
}

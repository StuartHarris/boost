// inspired by @fasterthanlime's brilliant post https://fasterthanli.me/articles/a-terminal-case-of-linux
// and Jakub Kądziołka's great follow up https://compilercrim.es/amos-nerdsniped-me/

use crate::cache;
use color_eyre::eyre::{bail, Result};
use once_cell::sync::OnceCell;
use std::{
    convert::TryFrom,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};
use tokio::{io::AsyncReadExt, process::Command, runtime::Runtime};
use tokio_fd::AsyncFd;

static RUNNER: OnceCell<CommandRunner> = OnceCell::new();

pub struct CommandRunner {
    runtime: Runtime,
}

impl CommandRunner {
    /// Initializes the global [`CommandRunner`] instance.
    pub fn init() -> &'static Self {
        RUNNER.get_or_init(|| Self {
            runtime: tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("Could not build tokio runtime"),
        })
    }

    /// Gets the global [`CommandRunner`] instance.
    ///
    /// # Panics
    /// Panics if the CommandRunner has been initialized yet.
    pub fn get() -> &'static Self {
        RUNNER.get().expect(
            "A CommandRunner has not been initialized yet. Please call \
                    CommandRunner::init beforehand.",
        )
    }

    pub(crate) async fn run(&self, command: &str, cache_dir: &Path) -> Result<()> {
        let command = command.to_owned();
        let cache_dir = cache_dir.to_owned();
        self.runtime.block_on(run(command, cache_dir))?;
        Ok(())
    }
}

async fn run(command: String, cache_dir: PathBuf) -> Result<()> {
    let (primary_fd, secondary_fd) = open_terminal();

    let mut cmd = Command::new("/bin/sh");
    cmd.args(["-c", &command]);

    unsafe {
        cmd.pre_exec(move || {
            if libc::login_tty(secondary_fd) != 0 {
                panic!("couldn't set the controlling terminal or something");
            }
            Ok(())
        })
    };
    let mut child = cmd.spawn()?;

    let mut writer_colors = File::create(cache_dir.join(cache::OUTPUT_COLORS_TXT_FILE))?;
    let output_plain = File::create(cache_dir.join(cache::OUTPUT_PLAIN_TXT_FILE))?;
    let mut writer_plain = strip_ansi_escapes::Writer::new(output_plain);

    let mut buf = vec![0u8; 1024];
    let mut primary = AsyncFd::try_from(primary_fd)?;

    loop {
        tokio::select! {
            n = primary.read(&mut buf) => {
                let n = n?;
                let slice = &buf[..n];

                let s = std::str::from_utf8(slice)?;
                print!("{}", s);

                writer_colors.write_all(slice)?;
                writer_plain.write_all(slice)?;
            },

            status = child.wait() => {
                match status {
                    Ok(s) => {
                        if s.success() {
                            break;
                        } else {
                            bail!("command failed with {}", s)
                        }
                    }
                    Err(e) => bail!(e),
                }
            },
        }
    }

    println!();

    Ok(())
}

fn open_terminal() -> (i32, i32) {
    let mut primary_fd: i32 = -1;
    let mut secondary_fd: i32 = -1;
    unsafe {
        let ret = libc::openpty(
            &mut primary_fd,
            &mut secondary_fd,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        if ret != 0 {
            panic!("Failed to openpty!");
        }
    };
    (primary_fd, secondary_fd)
}

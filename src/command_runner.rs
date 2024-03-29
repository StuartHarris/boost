// inspired by @fasterthanlime's brilliant post https://fasterthanli.me/articles/a-terminal-case-of-linux
// and Jakub Kądziołka's great follow up https://compilercrim.es/amos-nerdsniped-me/

use crate::cache;
use color_eyre::eyre::{bail, Result};
use std::{convert::TryFrom, fs::File, io::Write, path::Path};
use tokio::{io::AsyncReadExt, process::Command};
use tokio_fd::AsyncFd;

pub async fn run(command: &str, cache_dir: &Path) -> Result<()> {
    let (primary_fd, secondary_fd) = open_terminal();

    let mut cmd = Command::new("/bin/sh");
    cmd.args(["-c", command]);

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

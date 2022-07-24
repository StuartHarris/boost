// inspired by @fasterthanlime's brilliant post https://fasterthanli.me/articles/a-terminal-case-of-linux
// and Jakub Kądziołka's great follow up https://compilercrim.es/amos-nerdsniped-me/

use color_eyre::eyre::Result;
use std::{convert::TryFrom, path::Path};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
};
use tokio_fd::AsyncFd;

pub async fn run(command: &str, output_path: &Path) -> Result<()> {
    let (primary_fd, secondary_fd) = openpty();

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

    let mut out = vec![];
    let mut buf = vec![0u8; 1024];
    let mut primary = AsyncFd::try_from(primary_fd)?;

    loop {
        tokio::select! {
            n = primary.read(&mut buf) => {
                let n = n?;
                let slice = &buf[..n];
                let s = std::str::from_utf8(slice)?;
                print!("{}", s);
                out.extend_from_slice(slice);
            },

            status = child.wait() => {
                status?;
                break
            },
        }
    }

    let mut output = File::create(output_path).await?;
    output.write_all(&out[..]).await?;
    Ok(())
}

fn openpty() -> (i32, i32) {
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

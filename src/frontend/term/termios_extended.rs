use std::io;
use termios::{Termios,cfsetspeed};

#[cfg(target_os = "linux")]
pub fn set_fastest_speed(termios: &mut Termios) -> io::Result<()> {
    cfsetspeed(termios, termios::os::linux::B4000000)
}

#[cfg(target_os = "macos")]
pub fn set_fastest_speed(termios: &mut Termios) -> io::Result<()> {
    cfsetspeed(termios, termios::os::macos::B230400)
}

#[cfg(target_os = "freebsd")]
pub fn set_fastest_speed(termios: &mut Termios) -> io::Result<()> {
    cfsetspeed(termios, termios::os::freebsd::B921600)
}

#[cfg(target_os = "openbsd")]
pub fn set_fastest_speed(termios: &mut Termios) -> io::Result<()> {
    cfsetspeed(termios, termios::os::openbsd::B921600)
}

#[cfg(target_os = "netbsd")]
pub fn set_fastest_speed(termios: &mut Termios) -> io::Result<()> {
    cfsetspeed(termios, termios::os::netbsd::B921600)
}

#[cfg(target_os = "dragonfly")]
pub fn set_fastest_speed(termios: &mut Termios) -> io::Result<()> {
    cfsetspeed(termios, termios::os::dragonfly::B230400)
}

#[cfg(target_os = "solaris")]
pub fn set_fastest_speed(termios: &mut Termios) -> io::Result<()> {
    cfsetspeed(termios, termios::os::solaris::B921600)
}

#[cfg(target_os = "illumos")]
pub fn set_fastest_speed(termios: &mut Termios) -> io::Result<()> {
    cfsetspeed(termios, termios::os::illumos::B921600)
}


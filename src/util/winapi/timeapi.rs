/*
    Chip8Emu - a Chip-8 emulator
    Copyright (C) 2021  Philip Damianik

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
 
use std::{mem::size_of, os::raw::{c_uint}};

type UINT = c_uint;

/// A return type to indicate possible failure through different error/success codes
type MMRESULT = UINT;
/// Successful
pub const MMSYSERR_NOERROR: MMRESULT = 0;
/// Cannot execute given task
pub const TIMERR_NOCANDO: MMRESULT = 1;
/// General error
pub const MMSYSERR_ERROR: MMRESULT = 1;

#[inline(always)]
fn handleErrors(result: MMRESULT) -> Result<(), MMRESULT> {
    match result {
        MMSYSERR_NOERROR => Ok(()),
        _ => Err(result),
    }
}

#[repr(packed)]
pub struct TIMECAPS {
    w_period_min: UINT,
    w_period_max: UINT,
}
type LPTIMECAPS = *mut TIMECAPS;

mod ffi {
    #[link(name = "Winmm", kind = "static")]
    extern {
    	pub fn timeBeginPeriod(uPeriod: super::UINT) -> super::MMRESULT;
    	pub fn timeEndPeriod(uPeriod: super::UINT) -> super::MMRESULT;
        pub fn timeGetDevCaps(ptc: super::LPTIMECAPS , cbtc: super::UINT) -> super::MMRESULT;
    }
}

/// A wrapper around the Windows API function [`timeBeginPeriod()`](https://docs.microsoft.com/en-us/windows/win32/api/timeapi/nf-timeapi-timebeginperiod).
/// Documentation from the Windows docs:
/// The timeBeginPeriod function requests a minimum resolution for periodic timers.
///
/// This function can be used to get more accurate sleep delays.
/// ### Parameters
///  - `uPeriod` - Minimum timer resolution, in milliseconds, for the application or device driver. A lower value specifies a higher (more accurate) resolution.
/// ### Return value
/// Returns `Ok(())` if successful or `Err(TIMERR_NOCANDO)` if the resolution specified in u Period is out of range
pub fn timeBeginPeriod(uPeriod: UINT) -> Result<(), MMRESULT> {
    unsafe {
        handleErrors(ffi::timeBeginPeriod(uPeriod))
    }
}

/// A wrapper around the Windows API function [`timeEndPeriod()`](https://docs.microsoft.com/en-us/windows/win32/api/timeapi/nf-timeapi-timeendperiod).
/// Documentation from the Windows docs:
/// Minimum timer resolution specified in the previous call to the [`timeBeginPeriod`] function.
/// ### Parameters
///  - `uPeriod` - Minimum timer resolution specified in the previous call to the [`timeBeginPeriod`] function.
/// ### Return value
/// Returns `Ok(())` if successful or `Err(TIMERR_NOCANDO)` if the resolution specified in u Period is out of range
pub fn timeEndPeriod(uPeriod: UINT) -> Result<(), MMRESULT> {
    unsafe {
        handleErrors(ffi::timeEndPeriod(uPeriod))
    }
}

/// A wrapper around the Windows API function [`timeGetDevCaps()`](https://docs.microsoft.com/en-us/windows/win32/api/timeapi/nf-timeapi-timegetdevcaps).
/// Documentation from the Windows docs:
/// The timeGetDevCaps function queries the timer device to determine its resolution.
///
/// Gets the minimum and maximum resolution of the timer
/// ### Parameters
///  - `ptc` a reference to the [`TIMECAPS`] struct to fill with the query results
/// ### Return value
/// Returns `Ok(())` if successful or
///  - `Err(MMSYSERR_ERROR)` for general errors
///  - `Err(TIMERR_NOCANDO)` to indicate that the TIMECAPS struct is invalid
pub fn timeGetDevCaps(ptc: &mut TIMECAPS) -> Result<(), MMRESULT> {
    unsafe {
        handleErrors(ffi::timeGetDevCaps(ptc as *mut TIMECAPS, size_of::<TIMECAPS>() as UINT))
    }
}

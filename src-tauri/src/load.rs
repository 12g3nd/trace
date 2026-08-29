use serde::Serialize;
use std::process::{Command, Stdio};

#[cfg(windows)]
use std::{thread, time::Duration};
#[cfg(windows)]
use windows::Win32::{
    Foundation::FILETIME,
    System::{
        SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX},
        Threading::GetSystemTimes,
    },
};

const BYTES_PER_GIB: f64 = 1024.0 * 1024.0 * 1024.0;

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadState {
    pub memory_used_gib: f64,
    pub memory_total_gib: f64,
    pub cpu_percent: f64,
}

#[derive(Clone, Copy, Debug)]
struct CpuTimes {
    idle: u64,
    total: u64,
}

#[derive(Default)]
pub struct LoadSampler {
    previous_cpu: Option<CpuTimes>,
}

impl LoadSampler {
    #[cfg(windows)]
    pub fn sample(&mut self) -> Result<LoadState, String> {
        let (memory_used_gib, memory_total_gib) = memory_usage()?;
        let mut current = cpu_times()?;

        // The first LOAD visit takes one short sample so it can show a useful
        // value immediately. Later calls use the natural two-second UI cadence.
        if self.previous_cpu.is_none() {
            thread::sleep(Duration::from_millis(100));
            self.previous_cpu = Some(current);
            current = cpu_times()?;
        }

        let cpu_percent = self
            .previous_cpu
            .map(|previous| cpu_usage_percent(previous, current))
            .unwrap_or(0.0);
        self.previous_cpu = Some(current);

        Ok(LoadState {
            memory_used_gib,
            memory_total_gib,
            cpu_percent,
        })
    }

    #[cfg(not(windows))]
    pub fn sample(&mut self) -> Result<LoadState, String> {
        Err("system load metrics are only available on Windows".into())
    }
}

#[cfg(windows)]
fn memory_usage() -> Result<(f64, f64), String> {
    let mut status = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };
    unsafe { GlobalMemoryStatusEx(&mut status) }.map_err(|error| error.to_string())?;

    let used = status.ullTotalPhys.saturating_sub(status.ullAvailPhys);
    Ok((
        used as f64 / BYTES_PER_GIB,
        status.ullTotalPhys as f64 / BYTES_PER_GIB,
    ))
}

#[cfg(windows)]
fn cpu_times() -> Result<CpuTimes, String> {
    let mut idle = FILETIME::default();
    let mut kernel = FILETIME::default();
    let mut user = FILETIME::default();
    unsafe { GetSystemTimes(Some(&mut idle), Some(&mut kernel), Some(&mut user)) }
        .map_err(|error| error.to_string())?;

    let idle = filetime_to_u64(idle);
    let kernel = filetime_to_u64(kernel);
    let user = filetime_to_u64(user);
    Ok(CpuTimes {
        idle,
        total: kernel.saturating_add(user),
    })
}

#[cfg(windows)]
fn filetime_to_u64(value: FILETIME) -> u64 {
    ((value.dwHighDateTime as u64) << 32) | value.dwLowDateTime as u64
}

fn cpu_usage_percent(previous: CpuTimes, current: CpuTimes) -> f64 {
    let total_delta = current.total.saturating_sub(previous.total);
    if total_delta == 0 {
        return 0.0;
    }

    let idle_delta = current.idle.saturating_sub(previous.idle).min(total_delta);
    ((total_delta - idle_delta) as f64 / total_delta as f64 * 100.0).clamp(0.0, 100.0)
}

pub fn open_task_manager() -> Result<(), String> {
    let mut command = Command::new("taskmgr.exe");
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }

    command
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("failed to open Task Manager: {error}"))
}

#[cfg(test)]
mod tests {
    use super::{cpu_usage_percent, CpuTimes, BYTES_PER_GIB};

    #[test]
    fn computes_cpu_usage_from_system_time_deltas() {
        let previous = CpuTimes {
            idle: 100,
            total: 400,
        };
        let current = CpuTimes {
            idle: 130,
            total: 500,
        };
        assert_eq!(cpu_usage_percent(previous, current), 70.0);
    }

    #[test]
    fn clamps_invalid_or_empty_cpu_samples() {
        let sample = CpuTimes {
            idle: 100,
            total: 200,
        };
        assert_eq!(cpu_usage_percent(sample, sample), 0.0);
        assert_eq!(BYTES_PER_GIB, 1_073_741_824.0);
    }
}

use crate::error::AppResult;
use sysinfo::{ProcessesToUpdate, System};

pub fn list_processes(limit: usize) -> AppResult<String> {
    let limit = limit.clamp(1, 80);
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    // Second refresh so CPU percentages are meaningful.
    std::thread::sleep(std::time::Duration::from_millis(120));
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let mut rows: Vec<(f32, u32, String, u64)> = sys
        .processes()
        .iter()
        .map(|(pid, proc)| {
            (
                proc.cpu_usage(),
                pid.as_u32(),
                proc.name().to_string_lossy().into_owned(),
                proc.memory(),
            )
        })
        .collect();
    rows.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut lines = vec!["CPU%   PID     MEM(MB)  NOMBRE".into()];
    for (cpu, pid, name, mem) in rows.into_iter().take(limit) {
        lines.push(format!(
            "{cpu:5.1}  {pid:<7} {:>7.1}  {name}",
            mem as f64 / 1024.0 / 1024.0
        ));
    }
    Ok(lines.join("\n"))
}

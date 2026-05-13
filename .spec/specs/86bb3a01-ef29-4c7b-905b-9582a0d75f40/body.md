# viewer-ctl/process-management

viewer-ctl needs to find and terminate processes that occupy a TCP port.
The implementation in `src/process.rs` favours the most reliable tool
available on each platform and falls back gracefully when one is missing
or returns no results.

---

## `pids_on_port(port: u16) -> Vec<Pid>`

Tool selection priority:

| Platform        | First                                       | Then  | Then       |
|-----------------|---------------------------------------------|-------|------------|
| Windows         | PowerShell `Get-NetTCPConnection` (locale-agnostic) | `ss` | `netstat`  |
| Linux / macOS   | `ss -ltnp sport = :<port>`                  | `lsof -ti :<port> -sTCP:LISTEN` | `netstat -ano` |

The PowerShell path is preferred on Windows because `netstat`'s output is
locale-dependent (the "LISTENING" string is localised on non-English
installs), whereas `Get-NetTCPConnection -State Listen` filters on a
machine-readable enum value.

The `netstat` parser uses a locale-agnostic listening detection: a row's
foreign-address column ending in `:0` indicates a listener. PIDs are
deduplicated and sorted before return.

---

## `kill_process(pid: Pid, tag: &str) -> bool`

Increasingly aggressive escalation:

1. **Windows only:** `taskkill /F /PID <pid>`. Wait 300 ms, check
   liveness via `sysinfo`. Return `true` if gone.
2. **All platforms:** `kill <pid>` (SIGTERM). Wait 500 ms, check.
3. `kill -9 <pid>` (SIGKILL). Wait 500 ms, check, return result.

Liveness is checked through `sysinfo::System::process()`, which works
uniformly on every supported platform.

---

## `process_exists(pid: Pid) -> bool`

A thin wrapper around `sysinfo` that refreshes only the process list (not
all hardware sensors), keeping the call cheap.

---

## `print_process_info(pid: Pid, tag: &str)`

Best-effort identification of a doomed process. Tries:

- `tasklist /FI "PID eq <pid>" /FO LIST` (Windows; filters to image name,
  PID, mem-usage lines).
- `ps -p <pid> -o pid,comm,args --no-headers` (Unix).
- Falls back to printing just `PID: <pid>` so the user always knows what
  was killed.

---

## Acceptance Criteria

- `pids_on_port` returns an empty vector — never an error — when no tool
  on the platform can identify the listener.
- `pids_on_port` works on a Windows install with non-English UI language
  (PowerShell path is preferred precisely for this reason).
- `kill_process` is monotonic: the function only returns `true` when the
  process is genuinely gone, and the escalation never skips a step.
- `print_process_info` never panics, even for stale PIDs that no longer
  exist.

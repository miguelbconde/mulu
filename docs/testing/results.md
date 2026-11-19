# Test results

Every version is tested from scratch: download, install, first launch. Warnings, time and clicks are recorded here, with the system they were seen on. Items not reported are marked as such, never assumed.

## 0.0.1 (rehearsal), 2026-11-19

**Windows 11 Pro, maintainer's laptop. Tested directly on Windows, not in Windows Sandbox.**

- Download, installation and first launch: worked, described as quick and smooth.
- Window: interface in Brazilian Portuguese (detected from the system), 目录 seal and "Versão 0.0.1" shown, the "Choose the folder" button disabled as intended. Resizing kept the header and footer in place.
- Warnings from the browser or from Windows (SmartScreen): no warning that disrupted the installation was reported; exact texts not reported.
- Administrator password: not reported.
- Time and number of clicks: not measured.
- Reopening from the Start menu and uninstalling: not reported.

**Windows Sandbox (decision D12): failed.** The `.wsb` file attached to 0.0.1 was rejected ("Value does not fall within the expected range", an invalid configuration file); it is fixed for the next version. Windows Sandbox then also failed to start on its own from the Start menu, so the problem is in that computer's Windows Sandbox, not in Mulu. Because 0.0.1 reads no files and makes no network connections, the maintainer chose to install it directly. Windows Sandbox must work, or an alternative must be agreed, before testing 0.1, which reads the user's files.

**macOS:** not tested yet (no volunteer).

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

## 0.2.1, 2026-11-23

**Windows 11 Pro, maintainer's laptop, test account "Mulu Teste", test library in Public Documents (3,292 supported files, 13.7 GB). Installed over the previous version.**

- First reading: 24 min 37 s (1,477 s), 2,870 files read with text, 422 not read (no text 407, corrupted 12, protected 3). For comparison, 0.1.0 took about 1 h 10 min on the same library. 8 logical processors (4 cores); 3 files read at once in the background, 4 at full speed (used). Time in the reading processes 5,485 s; adding to the index 19 s. Slowest file: a 46.8 MB PDF with 7,206 sections, 123 s.
- Search `homero`: 8,268 passages in 574 files, 0.09 s (as expected).
- Libraries panel on the left: works.
- **Problem:** the folders inside the library open and close, but the library's top folder cannot be closed, so its whole list of folders stays on screen. Fixed for the next version.
- Changes (step 4): a new and a removed book, then a renamed and moved one, were each reported as expected.
- Second library (step 5): worked; the library a result comes from is visible only in the folder line under its name. Kept as is for now (a search can already be narrowed to one library by unticking the others).
- Refusal of a folder inside a library (step 6) and taking a library out without touching its files (step 7): as expected.
- Whole computer (step 8): counting took about 3 s and found 5,871 book files (31.1 GB). **Problem:** the "Before reading" window had "Read" and "Cancel" but no × at the top. Fixed for the next version. Places listed: not reported.
- Windows warnings (step 9): none.

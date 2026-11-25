# NPIN

NPIN is a Windows app that shows folders you choose as one 3D cloud. Each folder is a project. Dots are the project, its code, its notes, and things it uses, such as a package or a skill. If two folders use the same package, skill, API, or database, that is one shared dot.

Everything stays on this PC.

## Open the window

Do not double-click `app.exe`. The file from the last dev run is only a debug build. It loads the picture of the window from a local server, and that server stops when the terminal closes. Opening the file alone looks like a broken app.

The debug file, if it is still there, is not a shortcut you should keep:

`%LOCALAPPDATA%\Temp\cursor-sandbox-cache\100455e9ec8246e3c5d201e65fe2eafc\cargo-target\debug\app.exe`

Start it from the `app` folder instead. The first compile can take several minutes. A window titled **NPIN** opens when it finishes. Click **Add folders**.

```powershell
cd app
npm install
npm run tauri dev
```

Leave that terminal open while you use the window.

## If the compile says the linker is missing

The build needs three tools:

1. Node.js, so `npm` runs.
2. Rust, so `cargo` runs. The usual place is `%USERPROFILE%\.cargo\bin`.
3. Visual Studio with the workload **Desktop development with C++**. Rust uses `link.exe` from that workload. Without it, the error is `linker link.exe not found`.

A normal PowerShell window often does not see `cargo` or `link.exe`. From the `app` folder, paste this, then start the app:

```powershell
$cargo = Join-Path $env:USERPROFILE ".cargo\bin"
$msvcRoot = "C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC"
$msvc = Get-ChildItem $msvcRoot -Directory | Sort-Object Name -Descending | Select-Object -First 1
$sdk = "C:\Program Files (x86)\Windows Kits\10"
$kit = Get-ChildItem "$sdk\Lib" -Directory | Sort-Object Name -Descending | Select-Object -First 1
$env:PATH = "$cargo;$($msvc.FullName)\bin\Hostx64\x64;$sdk\bin\$($kit.Name)\x64;" + $env:PATH
$env:INCLUDE = "$($msvc.FullName)\include;$sdk\Include\$($kit.Name)\ucrt;$sdk\Include\$($kit.Name)\um;$sdk\Include\$($kit.Name)\shared;$sdk\Include\$($kit.Name)\winrt"
$env:LIB = "$($msvc.FullName)\lib\x64;$sdk\Lib\$($kit.Name)\ucrt\x64;$sdk\Lib\$($kit.Name)\um\x64"
npm run tauri dev
```

That picks the newest C++ tools already installed under Visual Studio 2026 Community. If the `MSVC` folder is missing, install the **Desktop development with C++** workload in the Visual Studio Installer, then run the snippet again.

## What the window does

- **Add folders** scans the folders you pick and draws them in one cloud.
- **Color → Project** paints each folder a different color. **Color → Kind** separates code, notes, and documents.
- Open folders stay watched. A file change scans that folder again.
- The top line, such as **Showing 2,226 dots from 7 folders**, opens the details: a short note, one row per folder, and **Saved**.
- **Save** keeps a skill or a rule. **Check this folder** shows what is missing. Files are written only after you confirm.
- Password files are not opened.

Scanning a folder also runs the local Graphify program, usually `%USERPROFILE%\.local\bin\graphify.exe`. If that program is missing, the window still opens. The folder row says the code map was not read.

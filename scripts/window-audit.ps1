<#
.SYNOPSIS
Runs one command the way the app runs its CLIs and reports whether anything it started opened a
terminal or a window, or took the foreground.

.DESCRIPTION
The app starts every child with CREATE_NO_WINDOW, piped stdio and no shell, but that covers only the
direct child: a grandchild a third-party CLI starts detached (a self-updater, an MCP server, a hook)
gets a console of its own, and Windows shows it (Windows Terminal takes the foreground). Tests
cannot see this, so after a CLI updates, run its preset here.

Records, until -Tail seconds after the command exits: every process it started (with descendants of
descendants), every console host the system started meanwhile (`OpenConsole -Embedding` is Windows
Terminal taking over a new console), every new visible top-level window and every foreground change.
Exit 1 when a new console window, a Windows Terminal window or a window of a descendant appeared.
Unrelated programs keep running while it records, so read the lines before blaming the command.

-Isolated runs the program from a throwaway profile (USERPROFILE, HOME, APPDATA, LOCALAPPDATA): a
self-updater checks at most every so often per profile, so a fresh one is what reproduces it. The
presets mirror the argv and environment in crates/collector-*/src/cli/system.rs and
crates/collector-codex/src/app_server.rs; keep them in step. -Isolated does not touch your profile;
without it the command runs against your real one, exactly like the app.

.EXAMPLE
./scripts/window-audit.ps1 -Preset agy -Isolated
./scripts/window-audit.ps1 -Program "$env:SystemRoot\System32\cmd.exe" -Arguments '/c', 'start', 'cmd', '/c', 'exit'
# From PowerShell call it with & or ./ : `pwsh -File` would turn -Arguments into one string.
#>
param(
    [ValidateSet('', 'claude', 'agy', 'codex')][string]$Preset = '',
    [string]$Program = '',
    [string[]]$Arguments = @(),
    [string[]]$Environment = @(),
    [string]$WorkingDirectory = '',
    [string]$Stdin = '',
    [double]$HoldStdin = 0,
    [double]$Tail = 20,
    [double]$Timeout = 60,
    [switch]$Isolated
)
$ErrorActionPreference = 'Stop'
$profileDir = $env:USERPROFILE
switch ($Preset) {
    'claude' {
        $Program = "$profileDir\.local\bin\claude.exe"
        $Arguments = '-p', '/usage', '--output-format', 'json', '--max-budget-usd', '0.02',
            '--setting-sources', 'project', '--strict-mcp-config', '--no-session-persistence'
        $Environment += 'DISABLE_AUTOUPDATER=1'
    }
    'agy' {
        $Program = "$profileDir\AppData\Local\agy\bin\agy.exe"
        $Arguments = '-p', '/usage', '--output-format', 'json'
        $Environment += 'AGY_CLI_DISABLE_AUTO_UPDATE=true'
    }
    'codex' {
        # PATHEXT order, like the app: the npm shim `codex.cmd`, never `codex.ps1`.
        $Program = (Get-Command codex -CommandType Application -ErrorAction Stop | Select-Object -First 1).Source
        $Arguments = @('app-server')
        $WorkingDirectory = Split-Path $Program
        $Stdin = (@(
                '{"id":1,"method":"initialize","params":{"clientInfo":{"name":"ai-session-monitor","title":"AI Session Monitor","version":"0.0.0"}}}',
                '{"method":"initialized","params":{}}',
                '{"id":2,"method":"account/read","params":{"refreshToken":false}}',
                '{"id":3,"method":"account/rateLimits/read"}') -join "`n") + "`n"
        $HoldStdin = 6
    }
}
if (-not $Program) { throw 'Give -Preset or -Program.' }
if (-not (Test-Path $Program)) { throw "Not installed: $Program" }

$scratch = Join-Path ([IO.Path]::GetTempPath()) ("aism-window-audit-" + [guid]::NewGuid().ToString('n').Substring(0, 8))
New-Item -ItemType Directory -Force "$scratch\cwd" | Out-Null
if (-not $WorkingDirectory) { $WorkingDirectory = "$scratch\cwd" }
if ($Isolated) {
    $home2 = "$scratch\profile"
    New-Item -ItemType Directory -Force "$home2\AppData\Roaming", "$home2\AppData\Local" | Out-Null
    $Environment += "USERPROFILE=$home2", "HOME=$home2", "APPDATA=$home2\AppData\Roaming",
        "LOCALAPPDATA=$home2\AppData\Local", "CLAUDE_CONFIG_DIR=$home2\.claude", "CODEX_HOME=$home2\.codex"
    $full = [IO.Path]::GetFullPath($Program)
    # Same place under the throwaway profile, where an updater may look for itself. A .cmd shim
    # stays put: it finds its program relative to its own folder.
    if ($full.EndsWith('.exe') -and $full.StartsWith($profileDir + '\', [StringComparison]::OrdinalIgnoreCase)) {
        $copy = Join-Path $home2 $full.Substring($profileDir.Length + 1)
        New-Item -ItemType Directory -Force (Split-Path $copy) | Out-Null
        try { New-Item -ItemType HardLink -Path $copy -Target $full | Out-Null } catch { Copy-Item $full $copy }
        $Program = $copy
    }
}

Add-Type -Language CSharp -TypeDefinition @'
using System; using System.Collections.Generic; using System.Diagnostics; using System.Linq;
using System.Runtime.InteropServices; using System.Text; using System.Threading;
public static class AismWindowAudit {
    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)] struct PE32 {
        public uint dwSize, cntUsage, pid; public IntPtr heap; public uint module, threads, ppid; public int pri; public uint flags;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 260)] public string exe; }
    [DllImport("kernel32.dll")] static extern IntPtr CreateToolhelp32Snapshot(uint f, uint p);
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode)] static extern bool Process32FirstW(IntPtr s, ref PE32 e);
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode)] static extern bool Process32NextW(IntPtr s, ref PE32 e);
    [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr h);
    [DllImport("kernel32.dll")] static extern IntPtr OpenProcess(uint a, bool i, uint p);
    [DllImport("ntdll.dll")] static extern int NtQueryInformationProcess(IntPtr h, int c, IntPtr b, int l, out int r);
    delegate bool EnumProc(IntPtr h, IntPtr l);
    [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc cb, IntPtr l);
    [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
    [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint p);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetClassNameW(IntPtr h, StringBuilder s, int n);
    [DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
    static readonly string[] Hosts = { "conhost.exe", "openconsole.exe", "windowsterminal.exe" };

    static Dictionary<uint, PE32> Snapshot() {
        var map = new Dictionary<uint, PE32>(); IntPtr s = CreateToolhelp32Snapshot(2, 0);
        var e = new PE32 { dwSize = (uint)Marshal.SizeOf(typeof(PE32)) };
        if (Process32FirstW(s, ref e)) do { map[e.pid] = e; } while (Process32NextW(s, ref e));
        CloseHandle(s); return map; }
    static string CommandLine(uint pid) {
        IntPtr h = OpenProcess(0x1000, false, pid); if (h == IntPtr.Zero) return "?";
        IntPtr b = Marshal.AllocHGlobal(65536);
        try { int r; if (NtQueryInformationProcess(h, 60, b, 65536, out r) != 0) return "?";
              return Marshal.PtrToStringUni(Marshal.ReadIntPtr(b, IntPtr.Size), (ushort)Marshal.ReadInt16(b) / 2); }
        finally { Marshal.FreeHGlobal(b); CloseHandle(h); } }
    static HashSet<IntPtr> Windows() {
        var set = new HashSet<IntPtr>(); EnumWindows((h, l) => { if (IsWindowVisible(h)) set.Add(h); return true; }, IntPtr.Zero); return set; }
    static string Window(IntPtr h, Dictionary<uint, PE32> procs, out uint pid) {
        GetWindowThreadProcessId(h, out pid); var c = new StringBuilder(256); GetClassNameW(h, c, 256);
        PE32 p; return string.Format("pid={0} {1} class={2}", pid, procs.TryGetValue(pid, out p) ? p.exe : "?", c); }

    public static int Run(string exe, string[] args, string cwd, string[] env, string stdin, double hold,
                          double tail, double timeout, Action<string> say) {
        var clock = Stopwatch.StartNew(); Action<string> log = s => say(string.Format("[{0,7:F2}s] {1}", clock.Elapsed.TotalSeconds, s));
        var known = new HashSet<uint>(Snapshot().Keys); var windows = Windows(); IntPtr fg = GetForegroundWindow();
        var psi = new ProcessStartInfo(exe) { UseShellExecute = false, CreateNoWindow = true, WorkingDirectory = cwd,
            RedirectStandardInput = true, RedirectStandardOutput = true, RedirectStandardError = true };
        foreach (var a in args) psi.ArgumentList.Add(a);
        foreach (var kv in env) { int i = kv.IndexOf('='); psi.Environment[kv.Substring(0, i)] = kv.Substring(i + 1); }
        var child = Process.Start(psi);
        new Thread(() => { try { child.StandardInput.Write(stdin); child.StandardInput.Flush(); Thread.Sleep((int)(hold * 1000)); child.StandardInput.Close(); } catch { } }) { IsBackground = true }.Start();
        new Thread(() => child.StandardOutput.ReadToEnd()) { IsBackground = true }.Start();
        new Thread(() => child.StandardError.ReadToEnd()) { IsBackground = true }.Start();
        var tree = new HashSet<uint> { (uint)child.Id }; known.Add((uint)child.Id);
        log("started pid=" + child.Id); double exited = -1; int bad = 0;
        while (true) {
            var procs = Snapshot();
            foreach (var p in procs.Values.OrderBy(p => p.pid)) {
                if (!known.Add(p.pid)) continue;
                bool mine = tree.Contains(p.ppid); if (mine) tree.Add(p.pid);
                // Every console program has a conhost; only a terminal taking a console over matters.
                bool host = Hosts.Skip(1).Contains(p.exe.ToLowerInvariant());
                if (!mine && !host) continue;
                string line = CommandLine(p.pid);
                if (p.exe.Equals("OpenConsole.exe", StringComparison.OrdinalIgnoreCase) && line.Contains("-Embedding")) { bad++; log("NEW CONSOLE handed to Windows Terminal: " + line); }
                else log(string.Format("{0} pid={1} parent={2} {3} :: {4}", mine ? "child" : "host ", p.pid, p.ppid, p.exe, line)); }
            foreach (var h in Windows()) {
                if (!windows.Add(h)) continue; uint pid; string what = Window(h, procs, out pid);
                PE32 owner; bool hostWindow = procs.TryGetValue(pid, out owner) && Hosts.Contains(owner.exe.ToLowerInvariant());
                if (tree.Contains(pid) || hostWindow) { bad++; log("NEW WINDOW " + what); } else log("new window (not ours) " + what); }
            IntPtr now = GetForegroundWindow();
            if (now != fg) { fg = now; uint pid; string what = Window(now, procs, out pid); log((tree.Contains(pid) ? "FOREGROUND TAKEN by " : "foreground -> ") + what); if (tree.Contains(pid)) bad++; }
            if (exited < 0 && child.HasExited) { exited = clock.Elapsed.TotalSeconds; log("exited code=" + child.ExitCode); }
            if (exited < 0 && clock.Elapsed.TotalSeconds > timeout) { log("timeout: killing the tree"); child.Kill(true); }
            if (exited >= 0 && clock.Elapsed.TotalSeconds > exited + tail) break;
            Thread.Sleep(5); }
        var alive = Snapshot(); foreach (var pid in tree) if (alive.ContainsKey(pid)) log("still running after the tail: pid=" + pid + " " + alive[pid].exe);
        return bad; }
}
'@

$redact = { param($line) $line.Replace($profileDir, '~') }
Write-Host "program: $(& $redact $Program) $($Arguments -join ' ')"
$found = [AismWindowAudit]::Run($Program, $Arguments, $WorkingDirectory, $Environment, $Stdin, $HoldStdin, $Tail, $Timeout,
    [Action[string]] { param($line) Write-Host (& $redact $line) })
Remove-Item -Recurse -Force $scratch -ErrorAction SilentlyContinue
if ($found -gt 0) { "VERDICT: $found window event(s) from this command"; exit 1 }
'VERDICT: no console, no window, no focus change from this command'

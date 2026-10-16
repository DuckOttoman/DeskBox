param([switch]$Crash,[switch]$CaptureDock)
$ErrorActionPreference='Stop'
Add-Type @'
using System;using System.Collections.Generic;using System.Runtime.InteropServices;
public static class QuickbarProbe {
 [StructLayout(LayoutKind.Sequential)]public struct Rect{public int l,t,r,b;}
 [StructLayout(LayoutKind.Sequential)]struct Bar{public uint size;public IntPtr hwnd;public uint callback,edge;public Rect rect;public IntPtr param;}
 [StructLayout(LayoutKind.Sequential)]struct Info{public uint size;public Rect monitor,work;public uint flags;}
 delegate bool EnumProc(IntPtr h,IntPtr p);
 [DllImport("user32.dll")]static extern bool EnumWindows(EnumProc f,IntPtr p);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)]static extern int GetClassName(IntPtr h,System.Text.StringBuilder s,int n);
 [DllImport("user32.dll")]static extern bool IsWindowVisible(IntPtr h);
 [DllImport("user32.dll")]static extern IntPtr MonitorFromWindow(IntPtr h,uint flags);
 [DllImport("user32.dll")]static extern bool GetMonitorInfo(IntPtr h,ref Info i);
 [DllImport("user32.dll")]static extern bool ShowWindow(IntPtr h,int command);
 [DllImport("shell32.dll")]static extern UIntPtr SHAppBarMessage(uint message,ref Bar b);
 public class Task {public long hwnd;public string kind;public bool visible;public int[] monitor,work;}
 public static Task[] Tasks(){var list=new List<Task>();EnumWindows((h,p)=>{var s=new System.Text.StringBuilder(128);GetClassName(h,s,128);var c=s.ToString();if(c=="Shell_TrayWnd"||c=="Shell_SecondaryTrayWnd"){var i=new Info();i.size=(uint)Marshal.SizeOf(i);GetMonitorInfo(MonitorFromWindow(h,2),ref i);list.Add(new Task{hwnd=h.ToInt64(),kind=c,visible=IsWindowVisible(h),monitor=new[]{i.monitor.l,i.monitor.t,i.monitor.r,i.monitor.b},work=new[]{i.work.l,i.work.t,i.work.r,i.work.b}});}return true;},IntPtr.Zero);return list.ToArray();}
 [DllImport("user32.dll",CharSet=CharSet.Unicode)]static extern IntPtr FindWindow(string cls,string title);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)]static extern IntPtr CreateWindowEx(uint ex,string cls,string title,uint style,int x,int y,int w,int h,IntPtr parent,IntPtr menu,IntPtr instance,IntPtr param);
 [DllImport("user32.dll")]static extern bool SetWindowPos(IntPtr h,IntPtr after,int x,int y,int w,int height,uint flags);
 [DllImport("user32.dll")]static extern bool DestroyWindow(IntPtr h);
 [DllImport("user32.dll")]static extern bool GetWindowRect(IntPtr h,out Rect rect);
 public static int[] DockRect(){Rect r;GetWindowRect(FindWindow(null,"DeskBox 快捷访问栏"),out r);return new[]{r.l,r.t,r.r,r.b};}
 public static bool DockVisible(){return IsWindowVisible(FindWindow(null,"DeskBox 快捷访问栏"));}
 public static IntPtr Fullscreen(int[] rect){var h=CreateWindowEx(0,"STATIC","DeskBox fullscreen diagnostic",0x90000000,rect[0],rect[1],rect[2]-rect[0],rect[3]-rect[1],IntPtr.Zero,IntPtr.Zero,IntPtr.Zero,IntPtr.Zero);if(h==IntPtr.Zero)throw new Exception("Cannot create diagnostic window");SetWindowPos(h,new IntPtr(-1),rect[0],rect[1],rect[2]-rect[0],rect[3]-rect[1],0x10);return h;}
 public static void CloseFullscreen(IntPtr h){DestroyWindow(h);}
 public static ulong State(){var b=new Bar();b.size=(uint)Marshal.SizeOf(b);return SHAppBarMessage(4,ref b).ToUInt64();}
 public static void Restore(ulong state,Task[] tasks){var b=new Bar();b.size=(uint)Marshal.SizeOf(b);b.param=new IntPtr((long)state);SHAppBarMessage(10,ref b);foreach(var t in tasks)ShowWindow(new IntPtr(t.hwnd),t.visible?4:0);}
}
'@
$root=Split-Path $PSScriptRoot -Parent
$case=if($Crash){'crash'}else{'normal'}
$dir=Join-Path $root ('evidence/quickbar-native-'+$case+'-'+(Get-Date -Format 'HHmmss'))
New-Item -ItemType Directory -Path $dir -Force | Out-Null
$before=[QuickbarProbe]::Tasks();$state=[QuickbarProbe]::State()
@{tasks=$before;state=$state}|ConvertTo-Json -Depth 8|Set-Content (Join-Path $dir 'before.json') -Encoding utf8
$source=Get-Content (Join-Path $root 'tests/quickbar-native-webview.js') -Raw -Encoding utf8
if($Crash){$source='window.__DB_CRASH_TEST=true;'+$source}
$script=Join-Path $dir 'driver.js';[IO.File]::WriteAllText($script,$source,[Text.UTF8Encoding]::new($false))
$env:DESKBOX_DATA_DIR=$dir;$env:DESKBOX_MERGE_CHECK_SCRIPT=$script
$p=$null;$fullscreenChecked=$false;$captured=$false;$dragCaptured=$false
try{
 $p=Start-Process (Join-Path $root 'target/release/deskbox-reconstructed.exe') -ArgumentList '--fence-merge-check','--drag-session','--quickbar-native-check' -WindowStyle Hidden -RedirectStandardError (Join-Path $dir 'stderr.log') -RedirectStandardOutput (Join-Path $dir 'stdout.log') -PassThru
 $deadline=(Get-Date).AddSeconds(45);$log=Join-Path $dir 'runtime.log'
 while((Get-Date)-lt $deadline -and -not $p.HasExited){
  $content=if(Test-Path $log){Get-Content $log -Raw -Encoding utf8}else{''}
  if($content -match 'FRONTEND_ERROR'){throw $content}
  if($CaptureDock -and -not $dragCaptured -and $content -match 'QUICKBAR_DRAG_CAPTURE_READY'){
   $screen=($before|Where-Object {$_.kind-eq 'Shell_TrayWnd'}|Select-Object -First 1).monitor
   $sx=[int](($screen[0]+$screen[2])/2-250);$sy=$screen[3]-110
   & 'C:/Users/Liang/.codex/skills/screenshot/scripts/take_screenshot.ps1' -Mode temp -Region "$sx,$sy,500,110" | Set-Content (Join-Path $dir 'drag-capture-path.txt') -Encoding utf8
   $dragCaptured=$true
  }
  if($CaptureDock -and -not $captured -and $content -match 'QUICKBAR_POLISH_CAPTURE_READY'){
   $screen=($before|Where-Object {$_.kind-eq 'Shell_TrayWnd'}|Select-Object -First 1).monitor
   $sx=[int](($screen[0]+$screen[2])/2-250);$sy=$screen[3]-110
   & 'C:/Users/Liang/.codex/skills/screenshot/scripts/take_screenshot.ps1' -Mode temp -Region "$sx,$sy,500,110" | Set-Content (Join-Path $dir 'capture-path.txt') -Encoding utf8
   $captured=$true
  }
  if(-not $Crash -and -not $fullscreenChecked -and $content -match 'QUICKBAR_REFERENCE_AND_PROTECTION_SUCCESS'){
   $dockRect=[QuickbarProbe]::DockRect();$primary=($before|Where-Object {$_.kind-eq 'Shell_TrayWnd'}|Select-Object -First 1).monitor
   @{dock=$dockRect;monitor=$primary}|ConvertTo-Json -Depth 4|Set-Content (Join-Path $dir 'dock-position.json') -Encoding utf8
   if([Math]::Abs($dockRect[3]-$primary[3])-gt 2){throw ('Dock shifted above screen bottom: '+($dockRect -join ',')+' expected bottom '+$primary[3])}
   $f=[QuickbarProbe]::Fullscreen(($before|Where-Object {$_.kind-eq 'Shell_TrayWnd'}|Select-Object -First 1).monitor)
   try{$until=(Get-Date).AddSeconds(2);while([QuickbarProbe]::DockVisible() -and (Get-Date)-lt $until){Start-Sleep -Milliseconds 50};if([QuickbarProbe]::DockVisible()){throw 'Dock remained visible over fullscreen'}}finally{[QuickbarProbe]::CloseFullscreen($f)}
   $until=(Get-Date).AddSeconds(2);while(-not [QuickbarProbe]::DockVisible() -and (Get-Date)-lt $until){Start-Sleep -Milliseconds 50};if(-not [QuickbarProbe]::DockVisible()){throw 'Dock did not return after fullscreen'}
   $returned=[QuickbarProbe]::DockRect();if([Math]::Abs($returned[3]-$primary[3])-gt 2){throw 'Dock shifted after fullscreen'}
   $fullscreenChecked=$true;'FULLSCREEN_HIDE_RESTORE_PASS'|Set-Content (Join-Path $dir 'fullscreen.txt')
  }
  if($Crash -and $content -match 'QUICKBAR_REFERENCE_AND_PROTECTION_SUCCESS'){Stop-Process -Id $p.Id -Force;break}
  Start-Sleep -Milliseconds 150
 }
 if(-not $p.WaitForExit(20000)){throw 'Quickbar test main did not exit'}
 $deadline=(Get-Date).AddSeconds(8)
 while((Test-Path (Join-Path $dir 'quickbar-recovery.json')) -and (Get-Date)-lt $deadline){Start-Sleep -Milliseconds 100}
 $content=Get-Content $log -Raw -Encoding utf8
 ('Main exit code: '+$p.ExitCode)|Set-Content (Join-Path $dir 'exit.txt')
 if($content -match 'FRONTEND_ERROR' -or $content -notmatch 'QUICKBAR_REFERENCE_AND_PROTECTION_SUCCESS'){throw $content}
 if($content -notmatch 'QUICKBAR_DRAG_PREVIEW_RENDERED'){throw 'Dock-window drag preview was not verified'}
 if(-not $Crash -and -not $fullscreenChecked){throw 'Fullscreen sequence not tested'}
 if(-not $Crash -and $content -notmatch 'QUICKBAR_REENABLED_FOR_EXIT'){throw 'Normal enable/disable/exit sequence incomplete'}
 $after=[QuickbarProbe]::Tasks();$afterState=[QuickbarProbe]::State()
 @{tasks=$after;state=$afterState}|ConvertTo-Json -Depth 8|Set-Content (Join-Path $dir 'after.json') -Encoding utf8
 if($state-ne $afterState){throw 'Auto-hide flags not restored'}
 foreach($t in $before){$a=$after|Where-Object {$_.kind-eq $t.kind -and ($_.monitor -join ',')-eq($t.monitor -join ',')}|Select-Object -First 1;if(-not $a -or $a.visible-ne $t.visible -or ($a.work -join ',')-ne($t.work -join ',')){throw 'Taskbar visibility/work area not restored'}}
 if(Test-Path (Join-Path $dir 'quickbar-recovery.json')){throw 'Recovery journal remained'}
 'PASS '+$case+' '+$dir
}finally{
 if($p -and -not $p.HasExited){Stop-Process -Id $p.Id -Force;Start-Sleep -Seconds 2}
 [QuickbarProbe]::Restore($state,$before)
 Remove-Item Env:DESKBOX_DATA_DIR -ErrorAction SilentlyContinue
 Remove-Item Env:DESKBOX_MERGE_CHECK_SCRIPT -ErrorAction SilentlyContinue
}

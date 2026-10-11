from pathlib import Path
import unittest
ROOT=Path(__file__).resolve().parents[1]
class InstallerContract(unittest.TestCase):
    def test_self_contained_and_scoped(self):
        s=(ROOT/'installer/deskbox.nsi').read_text(encoding='utf-8')
        for required in ['RequestExecutionLevel user','${RunningX64}','${AtLeastWin10}','WebView2Loader.dll','media-tools\\ffmpeg.exe','FFmpeg-LICENSE.txt','MicrosoftEdgeWebView2RuntimeInstallerX64.exe','/silent /install','WriteUninstaller','CreateShortcut','QuietUninstallString']:
            self.assertIn(required,s)
        self.assertNotIn('RMDir /r',s,'uninstall must not recursively delete user files')
        self.assertNotIn('taskkill',s.lower(),'installer must not force-kill running user app')
        self.assertNotIn('DeleteRegKey HKLM',s)
    def test_webview_detection_and_fail_closed(self):
        s=(ROOT/'installer/deskbox.nsi').read_text(encoding='utf-8')
        self.assertIn('F3017226-FE2A-4295-8BDF-00C3A9A7E4C5',s)
        self.assertIn('ReadRegStr $0 HKCU',s)
        self.assertIn('ReadRegStr $0 HKLM',s)
        self.assertIn('SetErrorLevel 1',s)
        self.assertIn('Call DetectWebView2',s)
if __name__=='__main__':unittest.main()

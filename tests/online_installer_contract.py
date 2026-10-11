from pathlib import Path
import unittest
ROOT=Path(__file__).resolve().parents[1]
class OnlineInstallerContract(unittest.TestCase):
    def test_verified_dependencies(self):
        s=(ROOT/'installer/download-dependencies.ps1').read_text(encoding='utf-8')
        for text in ['Get-FileHash','Get-AuthenticodeSignature','Microsoft Corporation','2124703','2ce797a0f88d7f067180338fb227f7b1928ea727bd9a4d7a1d022f7c52af71a3','Tls12']:
            self.assertIn(text,s)
    def test_online_distinct_output_and_conditional_payload(self):
        self.assertIn('--online',(ROOT/'tools/build_installer.py').read_text())
        self.assertIn('!ifdef ONLINE',(ROOT/'installer/deskbox.nsi').read_text(encoding='utf-8'))
if __name__=='__main__':unittest.main()

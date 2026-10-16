import importlib.util
import json
from pathlib import Path
import shutil
import unittest
from unittest import mock
import uuid
import zipfile
ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('package_plugin',ROOT/'tools/package_plugin.py')
p=importlib.util.module_from_spec(spec);spec.loader.exec_module(p)
class Packaging(unittest.TestCase):
 def setUp(self):
  self.tmp=ROOT/'tests'/('packaging-'+uuid.uuid4().hex);self.tmp.mkdir();self.addCleanup(lambda:shutil.rmtree(self.tmp))
  self.source=self.tmp/'source';self.source.mkdir();self.output=self.tmp/'plugin.deskbox-plugin'
  self.manifest=dict(apiVersion=1,id='org.example.clock',name='Clock',version='1.0.0',author='Example',type='widget',minDeskBoxVersion='0.1.0',entry='index.html');self.save();(self.source/'index.html').write_text('<p>Clock</p>')
 def save(self):(self.source/'manifest.json').write_text(json.dumps(self.manifest))
 def test_deterministic(self):
  p.package_plugin(self.source,self.output);before=self.output.read_bytes();p.package_plugin(self.source,self.output);self.assertEqual(before,self.output.read_bytes())
 def test_excludes_development_files(self):
  for folder in ['.git','node_modules']:(self.source/folder).mkdir();(self.source/folder/'private').write_text('secret')
  (self.source/'.env').write_text('secret');p.package_plugin(self.source,self.output)
  with zipfile.ZipFile(self.output) as z:self.assertEqual(set(z.namelist()),{'manifest.json','index.html'})
 def test_invalid_manifest(self):
  for key,value in [('id','../evil'),('apiVersion',2),('version','1.0'),('minDeskBoxVersion','99.0.0'),('entry','../x')]:
   original=self.manifest[key];self.manifest[key]=value;self.save()
   with self.assertRaises(ValueError):p.package_plugin(self.source,self.output)
   self.manifest[key]=original
 def test_bad_paths(self):
  for value in ['../x','/x','C:/x','a/CON.txt','a.exe','a:b','a./x','a\\b','a/CONIN$.txt','a/COM\u00b9.txt','a/LPT0','a/CON .txt']:
   with self.assertRaises(ValueError):p.validate_path(value)
 def test_collision_and_limits(self):
  with self.assertRaises(ValueError):p.validate_names(['index.html','INDEX.HTML'])
  with mock.patch.object(p,'MAX_FILE_BYTES',2):
   with self.assertRaises(ValueError):p.package_plugin(self.source,self.output)
 def test_aggregate_and_compressed_limits(self):
  for limit in ['MAX_TOTAL_BYTES','MAX_FILES','MAX_PACKAGE_BYTES']:
   with mock.patch.object(p,limit,1):
    with self.assertRaises(ValueError):p.package_plugin(self.source,self.output)
  self.assertFalse(self.output.exists())
 def test_permissions(self):
  for value in [{'network':['http://example.com']},{'network':['https://example.com/a']},{'network':'https://example.com'},{'systemMetrics':'true'}]:
   self.manifest['permissions']=value;self.save()
   with self.assertRaises(ValueError):p.package_plugin(self.source,self.output)
 def test_theme(self):
  p.validate_theme({'variables':{'--db-plugin-color':'#fff'}})
  for theme in [{'variables':{'color':'red'}},{'variables':{'--db-plugin-background':'url(https://evil.com)'}},{'variables':{'--db-plugin-color':'red;display:none'}}]:
   with self.assertRaises(ValueError):p.validate_theme(theme)
 def test_theme_background_asset(self):
  p.validate_theme({'variables':{},'backgroundImage':'assets/tile.png'},{'assets/tile.png':b'png'})
  for name in ['../tile.png','https://example.org/tile.png','assets/tile.svg','assets/missing.png']:
   with self.assertRaises(ValueError):p.validate_theme({'variables':{},'backgroundImage':name},{'assets/tile.png':b'png'})
  for value in ['u\\72l(https://example.org)','@import test','image-set("https://evil/x" 1x)','image ("https://evil/x")','src("https://evil/x")','u r l (https://evil/x)','rgb/**/(0,0,0)']:
   with self.assertRaises(ValueError):p.validate_theme({'variables':{'--db-plugin-color':value}})
 def test_symlink(self):
  try:(self.source/'linked').symlink_to(self.source/'index.html')
  except OSError:self.skipTest('symlink creation unavailable')
  with self.assertRaises(ValueError):p.package_plugin(self.source,self.output)
if __name__=='__main__':unittest.main()

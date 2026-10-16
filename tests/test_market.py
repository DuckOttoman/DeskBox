import importlib.util, pathlib, tempfile, unittest, threading, http.client, json
spec=importlib.util.spec_from_file_location('market',pathlib.Path(__file__).parents[1]/'server/market/market.py')
market=importlib.util.module_from_spec(spec);spec.loader.exec_module(market)
class MarketTests(unittest.TestCase):
 def test_catalog(self):
  self.assertEqual(market.validate_catalog({'schemaVersion':1,'entries':[]})['entries'],[])
 def test_bad_schema(self):
  with self.assertRaises(ValueError):market.validate_catalog({'schemaVersion':2,'entries':[]})
 def test_duplicate(self):
  e={'id':'org.example.clock','name':'Clock','author':'Example','description':'Clock','type':'widget','version':'1.0.0','url':'https://47.85.175.203/packages/clock.deskbox-plugin','sha256':'a'*64}
  with self.assertRaises(ValueError):market.validate_catalog({'schemaVersion':1,'entries':[e,e]})
 def test_no_http(self):
  e={'id':'org.example.clock','name':'Clock','author':'Example','description':'Clock','type':'widget','version':'1.0.0','url':'http://example.org/a','sha256':'a'*64}
  with self.assertRaises(ValueError):market.validate_catalog({'schemaVersion':1,'entries':[e]})
 def test_only_public_files(self):
  with tempfile.TemporaryDirectory(dir=pathlib.Path(__file__).parents[1]/"evidence") as d:
   root=pathlib.Path(d);(root/'catalog.json').write_text('{}');(root/'secret.txt').write_text('secret')
   self.assertEqual(market.public_path(root,'/catalog.json'),root/'catalog.json')
   for path in ['/secret.txt','/../catalog.json','/%2e%2e/catalog.json','/packages/../../secret.txt','/packages/a.exe']:
    self.assertIsNone(market.public_path(root,path))
 def test_symlink_escape(self):
  with tempfile.TemporaryDirectory(dir=pathlib.Path(__file__).parents[1]/"evidence") as d:
   root=pathlib.Path(d);(root/'packages').mkdir()
   try:(root/'packages/x.deskbox-plugin').symlink_to(root/'outside')
   except OSError:self.skipTest('symlink unavailable')
   self.assertIsNone(market.public_path(root,'/packages/x.deskbox-plugin'))
class HttpTests(unittest.TestCase):
 def test_readonly_routes(self):
  with tempfile.TemporaryDirectory(dir=pathlib.Path(__file__).parents[1]/"evidence") as d:
   root=pathlib.Path(d);(root/'packages').mkdir();(root/'catalog.json').write_text('{"schemaVersion":1,"entries":[]}')
   (root/'packages/demo.deskbox-plugin').write_bytes(b'example')
   server=market.Server(('127.0.0.1',0),market.Handler);server.root=root
   thread=threading.Thread(target=server.serve_forever,daemon=True);thread.start()
   try:
    def request(method,path):
     c=http.client.HTTPConnection('127.0.0.1',server.server_address[1],timeout=5);c.request(method,path);r=c.getresponse();result=(r.status,r.read(),dict(r.getheaders()));c.close();return result
    self.assertEqual(json.loads(request('GET','/health')[1])['status'],'ok')
    self.assertEqual(json.loads(request('GET','/catalog.json')[1])['schemaVersion'],1)
    self.assertEqual(request('GET','/packages/demo.deskbox-plugin')[1],b'example')
    self.assertEqual(request('HEAD','/packages/demo.deskbox-plugin')[1],b'')
    self.assertEqual(request('GET','/')[0],404)
    self.assertEqual(request('GET','/%2e%2e/market.py')[0],404)
    self.assertEqual(request('POST','/catalog.json')[0],501)
    (root/'catalog.json').write_text('{"schemaVersion":99,"entries":[]}')
    self.assertEqual(request('GET','/catalog.json')[0],503)
   finally:server.shutdown();server.server_close();thread.join(5)
if __name__=='__main__':unittest.main()


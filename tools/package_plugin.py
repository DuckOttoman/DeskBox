"""Validate and deterministically package a DeskBox API v1 plugin (stdlib only)."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import stat
import zipfile
from urllib.parse import urlsplit

MAX_PACKAGE_BYTES=20*1024*1024
MAX_TOTAL_BYTES=50*1024*1024
MAX_FILE_BYTES=10*1024*1024
MAX_FILES=512
EXCLUDED={'.git','.svn','.hg','node_modules','__pycache__','.venv','venv','.env','.DS_Store'}
EXECUTABLES={'.exe','.dll','.com','.bat','.cmd','.ps1','.vbs','.msi','.scr','.pif','.sys','.cpl','.jar','.hta','.sh','.lnk'}
RESERVED={'con','prn','aux','nul','conin$','conout$','com0','lpt0',*(prefix+suffix for prefix in ('com','lpt') for suffix in ('\u00b9','\u00b2','\u00b3')),*(f'com{i}' for i in range(1,10)),*(f'lpt{i}' for i in range(1,10))}

def validate_path(name):
    if not isinstance(name,str) or not name or '\\' in name or name.startswith('/') or len(name.encode("utf-8"))>240:
        raise ValueError('Invalid package path')
    for part in name.split('/'):
        if part in ('','.','..') or part.endswith((' ','.')) or any(c in part for c in '<>:"|?*') or any(ord(c)<32 for c in part) or part.split('.')[0].rstrip().lower() in RESERVED:
            raise ValueError(f'Unsafe package path: {name}')
    if Path(name).suffix.lower() in EXECUTABLES: raise ValueError(f'Executable file forbidden: {name}')
    return name

def validate_names(names):
    seen=set()
    for name in names:
        validate_path(name)
        if name.casefold() in seen: raise ValueError('Duplicate or case-colliding path')
        seen.add(name.casefold())

def version(value):
    if not isinstance(value,str) or not re.fullmatch(r'[0-9]+\.[0-9]+\.[0-9]+',value): raise ValueError('Version must have three numeric components')
    result=tuple(map(int,value.split('.')))
    if any(n>4294967295 for n in result): raise ValueError('Version component exceeds u32')
    return result

def https(value,origin=False):
    if not isinstance(value,str): raise ValueError('HTTPS URL required')
    u=urlsplit(value)
    if u.scheme!='https' or not u.hostname or u.username or u.password or u.fragment or (origin and (u.path not in ('','/') or u.query)):
        raise ValueError('HTTPS origin required' if origin else 'HTTPS URL required')
    try: u.port
    except ValueError: raise ValueError('Invalid port')

def validate_manifest(m,files,host_version='0.1.0'):
    if not isinstance(m,dict) or type(m.get('apiVersion')) is not int or m['apiVersion']!=1: raise ValueError('Unsupported apiVersion')
    if not isinstance(m.get('id'),str) or len(m['id'])>120 or not re.fullmatch(r'[a-z0-9]+(?:[a-z0-9-]*[a-z0-9])?(?:\.[a-z0-9]+(?:[a-z0-9-]*[a-z0-9])?)+',m['id']): raise ValueError('Invalid reverse-domain plugin id')
    for key in ('name','author'):
        if not isinstance(m.get(key),str) or not m[key].strip() or len(m[key].encode("utf-8"))>512: raise ValueError(f'Missing {key}')
    version(m.get('version'))
    if version(m.get('minDeskBoxVersion'))>version(host_version): raise ValueError('Plugin needs a newer DeskBox')
    if m.get('type') not in ('widget','theme'): raise ValueError('Invalid plugin type')
    entry=m.get('entry') if m['type']=='widget' else 'theme.json'
    validate_path(entry)
    if entry not in files: raise ValueError(f'Missing entry: {entry}')
    for key in ('homepage','updateUrl'):
        if key in m: https(m[key])
    permissions=m.get('permissions',{})
    if not isinstance(permissions,dict) or set(permissions)-{'network','systemMetrics'}: raise ValueError('Invalid permissions')
    if not isinstance(permissions.get('network',[]),list): raise ValueError('network must be an origin list')
    for origin in permissions.get('network',[]): https(origin,True)
    if 'systemMetrics' in permissions and type(permissions['systemMetrics']) is not bool: raise ValueError('systemMetrics must be boolean')
    return m

def validate_theme(theme,files=None):
    variables=theme.get('variables',{k:v for k,v in theme.items() if k!='backgroundImage'}) if isinstance(theme,dict) else None
    allowed={'--db-plugin-background','--db-plugin-color','--db-plugin-accent','--db-plugin-border','--db-plugin-radius','--db-plugin-shadow','--db-plugin-font'}
    if not isinstance(variables,dict): raise ValueError('Theme must be a variable map')
    functions={'rgb','rgba','hsl','hsla','oklch','lab','lch','oklab','linear-gradient','radial-gradient','conic-gradient','repeating-linear-gradient','repeating-radial-gradient','repeating-conic-gradient','calc','clamp','min','max','var','color','color-mix'}
    for key,value in variables.items():
        if key not in allowed or not isinstance(value,str) or len(value.encode("utf-8"))>512 or any(c in value for c in ';{}<>\\@') or '/*' in value or any(name.lower() not in functions for name in re.findall(r'([a-z-]+)\s*\(',value,re.I)) or any(token in ''.join(value.lower().split()) for token in ('url(','image-set(','image(','src(','expression')): raise ValueError('Unsafe or unsupported theme variable')

    if 'backgroundImage' in theme:
        image=theme['backgroundImage']
        validate_path(image)
        if Path(image).suffix.lower() not in {'.png','.jpg','.jpeg','.webp','.gif'} or image not in (files or {}): raise ValueError('Background image must be an existing raster package asset')

def package_plugin(source,output,host_version='0.1.0'):
    source,output=Path(source).resolve(),Path(output).resolve()
    if not source.is_dir(): raise ValueError('Source must be a directory')
    if output==source or source in output.parents: raise ValueError('Output must be outside source')
    files={}; total=0; seen=set()
    for item in sorted(source.rglob('*')):
        rel=item.relative_to(source)
        if any(part in EXCLUDED or part.startswith('.env.') for part in rel.parts): continue
        if item.is_symlink() or bool(getattr(item.lstat(),'st_file_attributes',0)&0x400): raise ValueError(f'Links forbidden: {rel}')
        if item.is_dir(): continue
        if not stat.S_ISREG(item.stat().st_mode): raise ValueError('Only regular files allowed')
        if item.resolve()==output: raise ValueError('Output cannot be inside source')
        name=validate_path(rel.as_posix())
        if name.casefold() in seen: raise ValueError('Case-colliding package paths')
        seen.add(name.casefold()); size=item.stat().st_size; total+=size
        if size>MAX_FILE_BYTES or total>MAX_TOTAL_BYTES or len(files)>=MAX_FILES: raise ValueError('Package content exceeds limits')
        files[name]=item.read_bytes()
    if 'manifest.json' not in files: raise ValueError('Root manifest.json required')
    m=validate_manifest(json.loads(files['manifest.json'].decode('utf-8-sig')),files,host_version)
    if m['type']=='theme': validate_theme(json.loads(files['theme.json'].decode('utf-8-sig')),files)
    output.parent.mkdir(parents=True,exist_ok=True)
    temporary=output.with_name(output.name+'.tmp')
    try:
        with zipfile.ZipFile(temporary,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=9) as archive:
            for name,data in sorted(files.items()):
                info=zipfile.ZipInfo(name,date_time=(1980,1,1,0,0,0)); info.compress_type=zipfile.ZIP_DEFLATED
                info.create_system=3; info.external_attr=(stat.S_IFREG|0o644)<<16
                archive.writestr(info,data,compresslevel=9)
        if temporary.stat().st_size>MAX_PACKAGE_BYTES: raise ValueError('Compressed package exceeds 20 MiB')
        temporary.replace(output)
    finally:
        temporary.unlink(missing_ok=True)
    return {'id':m['id'],'version':m['version'],'path':str(output),'sha256':hashlib.sha256(output.read_bytes()).hexdigest()}

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source',type=Path); parser.add_argument('output',type=Path); parser.add_argument('--host-version',default='0.1.0')
    args=parser.parse_args()
    try: result=package_plugin(args.source,args.output,args.host_version)
    except (ValueError,OSError,UnicodeError) as error: parser.exit(1,f'Packaging failed: {error}\n')
    print(json.dumps(result,ensure_ascii=False,indent=2))

if __name__=='__main__': main()





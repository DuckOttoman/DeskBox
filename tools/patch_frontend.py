"""Build the editable frontend variant while preserving original dist assets."""
import hashlib
import json
from pathlib import Path
import shutil

root = Path(__file__).resolve().parents[1]
output = root / 'frontend'
shutil.copytree(root / 'dist', output, dirs_exist_ok=True)
bundle = output / 'assets/index-B3Mcm4P1.js'
text = bundle.read_text(encoding='utf-8')
anchor = '{label:"属性",icon:"info",action:()=>te.itemAction(T.id,"properties")}'
assert text.count(anchor) == 1, 'Original menu anchor changed'
text = text.replace(anchor, '{label:"重命名",icon:"edit",disabled:le,action:()=>window.DeskBoxRename.open(T)},' + anchor)
# Preserve recovered entry/action semantics; replace only fixed menu placement.
start = text.index('__name:"PopMenu"')
computed_start = text.index('l=pe(()=>{const r={};', start)
computed_end = text.index('});function o(r,a)', computed_start) + 2
text = text[:computed_start] + 'l=pe(()=>window.DeskBoxMenuLayout.place({x:n.x,y:n.y,h:n.h,v:n.v},__dbMenuSize.value,__dbMenuViewport.value))' + text[computed_end:]
anchor = 'const n=e,s=t,i=_e(null),l=pe(()=>window.DeskBoxMenuLayout.place'
assert text.count(anchor) == 1
text = text.replace(anchor, 'const n=e,s=t,i=_e(null),__dbMenuNode=_e(null),__dbMenuSize=_e({width:260,height:380}),__dbMenuViewport=_e({width:window.innerWidth,height:window.innerHeight}),l=pe(()=>window.DeskBoxMenuLayout.place')
# The new computed expression ends in )), unlike the recovered callback's }).
anchor = '));function o(r,a){if(!(r.disabled||r.separator))'
assert text.count(anchor) == 1
lifecycle = '''let __dbMenuObserver;const __dbMenuResize=()=>{__dbMenuViewport.value={width:window.innerWidth,height:window.innerHeight}};In(()=>{__dbMenuObserver=new ResizeObserver(()=>{const el=__dbMenuNode.value;if(el){const width=el.offsetWidth,height=el.offsetHeight;if(width!==__dbMenuSize.value.width||height!==__dbMenuSize.value.height)__dbMenuSize.value={width,height}}});if(__dbMenuNode.value)__dbMenuObserver.observe(__dbMenuNode.value);window.addEventListener("resize",__dbMenuResize)});Ln(()=>{__dbMenuObserver?.disconnect();window.removeEventListener("resize",__dbMenuResize)});'''
text = text.replace(anchor, '));' + lifecycle + 'function o(r,a){if(!(r.disabled||r.separator))')
anchor = 'class:"pm","data-hit":"",style:we(l.value)'
assert text.count(anchor) == 1
text = text.replace(anchor, 'ref:__dbMenuNode,class:"pm","data-hit":"",style:we(l.value)')
fence_start = text.index('__name:"FenceBox"')
drag_start = text.index('function re($)', fence_start)
drag_end = text.index('function x($,T)', drag_start)
text = text[:drag_start] + 'function re($){if(a.value)return;window.DeskBoxFenceMerge.pointer($,i.value,n.snap)}' + text[drag_end:]
def replace_once(old, new):
    global text
    assert text.count(old) == 1, 'Merge hook changed: ' + old
    text = text.replace(old, new)
replace_once('left:`${i.value.x}px`,top:`${i.value.y}px`,width:`${i.value.w}px`', 'left:`${window.DeskBoxFenceMerge.geometry(i.value).x}px`,top:`${window.DeskBoxFenceMerge.geometry(i.value).y}px`,width:`${i.value.w}px`')
replace_once('W("span",Xu,q(i.value.title),1)', 'W("span",{key:2,class:"title",onClick:ev=>window.DeskBoxFenceMerge.title(ev,i.value)},q(i.value.title),8,["onClick"])')
replace_once('s("menu",$,T,i.value.title)', 's("menu",$,window.DeskBoxFenceMerge.menu(T,i.value),i.value.title)')
replace_once('.filter(x=>!x.hidden&&s(x.monitor))', '.filter(x=>!x.hidden&&s(x.monitor)&&window.DeskBoxFenceMerge.visible(x))')
fence_end = text.index('__name:"WidgetBox"', fence_start)
block = text[fence_start:fence_end]
assert block.count('resizing:r.value,collapsed:i.value.collapsed') == 1, 'Icon-name fence class hook changed'
block = block.replace('resizing:r.value,collapsed:i.value.collapsed', 'resizing:r.value,collapsed:i.value.collapsed,"db-hide-icon-names":!!i.value.style.hideIconNames')
block = block.replace('window.innerWidth-i.value.x', 'window.innerWidth-window.DeskBoxFenceMerge.geometry(i.value).x')
block = block.replace('window.innerHeight-i.value.y', 'window.innerHeight-window.DeskBoxFenceMerge.geometry(i.value).y')
block = block.replace('$.x-i.value.x+dl', '$.x-window.DeskBoxFenceMerge.geometry(i.value).x+dl')
block = block.replace('$.y-i.value.y+dl', '$.y-window.DeskBoxFenceMerge.geometry(i.value).y+dl')
text = text[:fence_start] + block + text[fence_end:]
text += '\nwindow.DeskBoxFenceMerge.bind({state:B,invoke:(command,args)=>window.__TAURI_INTERNALS__.invoke(command,args),hold:st,region:Ro,focus:te.keyboardFocus});\n'
text += '\nwindow.DeskBoxRename.bind({focus:te.keyboardFocus,hold:st,region:Ro});\n'
start = text.index('__name:"StylePanel"')
end = text.index('__name:"OverlayView"', start)
block = text[start:end]
def style_replace(old, new):
    global block
    assert block.count(old) == 1, 'Media style hook changed: ' + old
    block = block.replace(old, new)
style_replace('await te.pickFenceImage(o.value.id)', 'await window.DeskBoxMedia.pick(o.value)')
style_replace('Xe(te).clearFenceImage(o.value.id)', 'window.DeskBoxMedia.clear(o.value)')
style_replace('o.value.style={image:o.value.style.image}', 'o.value.style=window.DeskBoxMedia.resetStyle(o.value.style)')
style_replace('const g=pe(()=>({left:`${Math.min(n.x,window.innerWidth-290)}px`,top:`${Math.max(8,Math.min(n.y,window.innerHeight-560))}px`}));', 'const g=pe(()=>window.DeskBoxMedia.panelStyle(n.x,n.y));')
style_replace(']),u.value?(L(),W(ve,{key:0},[d("label"', ']),d("div",{class:"db-media-controls"},window.DeskBoxMedia.controls(d,o.value,a.value)),u.value?(L(),W(ve,{key:0},[d("label"')
block = block.replace('背景图片', '背景素材').replace('选择中…', '处理背景中…').replace('更换图片', '更换背景').replace('选择图片…', '选择背景…')
text = text[:start] + block + text[end:]
text += '\nwindow.DeskBoxMedia.bind({state:B,label:Rn().label,asset:An,invoke:(command,args)=>window.__TAURI_INTERNALS__.invoke(command,args)});\n'
# VRAM shares the memory header, numeric layout and usage bar.
replace_once('t.value.vramTotal?(L(),W("div",kc,[d("span",null,[d("b",null,q(s(t.value.vramUsed??0))+" / "+q(s(t.value.vramTotal))+" GB",1),c[9]||(c[9]=ue("显存",-1))])])):ye("",!0)', 't.value.vramTotal?(L(),W("div",{key:0,class:"db-vram"},[d("div",$c,[d("span",{class:"tag mem"},"显存"),d("span",Ac,q(window.DeskBoxVram.text(t.value)),1)]),d("div",Oc,[d("i",{class:be(r(window.DeskBoxVram.ratio(t.value)*100)),style:we({transform:`scaleX(${window.DeskBoxVram.ratio(t.value)})`})},null,6)])])):ye("",!0)')
# CPU temperature uses current per-core range, with a persisted per-widget mode.
replace_once('__name:"SysmonWidget",setup(e)', '__name:"SysmonWidget",props:{widget:{}},setup(e)')
replace_once('d("span",Sc,[d("b",null,q(o(t.value.cpuTemp)),1),c[3]||(c[3]=ue("温度",-1))])', 'window.DeskBoxCpuTemperature.view(d,e.widget,t.value)')
# Add editable widgets while retaining recovered native region/theme/system-monitor components.
start=text.index('__name:"WidgetBox"');end=text.index('__name:"StylePanel"',start)
block=text[start:end]
block=block.replace('let a=null;In(()=>{', 'const __dbContent=wt({props:{widget:{},tick:{}},setup(__p){return()=>window.DeskBoxWidgets.content((tag,props,children)=>d(tag,props,children,-2),__p.widget)}});const __dbTick=_e(0);let __dbClock=setInterval(()=>{window.DeskBoxWidgets.tick(i.value);__dbTick.value++},1000);let a=null;In(()=>{')
block=block.replace('Ln(()=>a?.disconnect());', 'Ln(()=>{a?.disconnect();clearInterval(__dbClock);window.DeskBoxWidgets.flush(i.value.id)});')
u=block.index('function u(g)');c=block.index('function c(g)',u)
block=block[:u]+'function u(g){window.DeskBoxWidgets.pointer(g,i.value,n.snap)}'+block[c:]
block=block.replace('s("menu",g,[','s("menu",g,window.DeskBoxWidgets.menu(i.value,[')
block=block.replace('],i.value.kind==="clock"?"时钟":"系统监控")', ']),window.DeskBoxWidgets.label(i.value))')
block=block.replace('"data-hit":"",style:we({left:', '"data-hit":"","data-widget":i.value.id,"data-shape":window.DeskBoxWidgets.form(window.DeskBoxWidgets.dimensions(i.value).width,window.DeskBoxWidgets.dimensions(i.value).height),style:we({"--widget-width":`${window.DeskBoxWidgets.dimensions(i.value).width}px`,"--widget-height":`${window.DeskBoxWidgets.dimensions(i.value).height}px`,left:')
old='d("div",Rc,[i.value.kind==="clock"?(L(),We(mc,{key:0})):(L(),We(Lc,{key:1}))])'
new='d("div",Rc,[d("header",{class:"db-widget-heading"},window.DeskBoxWidgets.label(i.value)),i.value.kind==="sysmon"?(L(),We(Lc,{key:1,widget:i.value},null,8,["widget"])):(L(),We(__dbContent,{widget:i.value,tick:__dbTick.value},null,8,["widget","tick"])),...window.DeskBoxWidgets.handles(d,i.value,n.snap)])'
assert block.count(old)==1,'Widget render anchor changed'
block=block.replace(old,new)
assert block.count('])],38))')==1,'Widget root patch flags changed'
block=block.replace('])],38))', '])],54))')
assert 'window.DeskBoxWidgets.label(i.value))' in block,'Widget menu hook changed'
text=text[:start]+block+text[end:]
old='{label:"系统监控",icon:"chip",action:async()=>te.createWidget("sysmon",await a())}'
new=old+',...[["timer","倒计时与番茄钟","clock"],["notes","便签与待办","edit"],["album","相册","folder"]].map(([kind,label,icon])=>({label,icon,action:async()=>te.createWidget(kind,await a())}))'
replace_once(old,new)
replace_once('i&&Object.assign(s,{x:i.x,y:i.y})','if(i){Object.assign(s,{x:i.x,y:i.y});for(const key of ["widgetWidth","widgetHeight"]){if(i.style[key]===undefined)delete s.style[key];else s.style[key]=i.style[key]}}')
replace_once('for(const s of e.config.widgets)if(B.busy.has(s.id))','for(const g of e.config.fenceGroups||[])if(B.busy.has(g.active)){const old=t.fenceGroups?.find(x=>x.id===g.id);if(old){g.x=old.x;g.y=old.y}}for(const s of e.config.widgets)if(B.busy.has(s.id))')
text+='\nwindow.DeskBoxWidgets.bind({listen:Xt,label:as,state:B,hold:st,region:Ro,focus:te.keyboardFocus,asset:An,invoke:(command,args)=>window.__TAURI_INTERNALS__.invoke(command,args)});\n'
replace_once('{label:"退出",icon:"power",danger:!0,action:()=>te.quit()}', '{label:"布局备份",icon:"folder",action:()=>window.__TAURI_INTERNALS__.invoke("request_layout_backup",{action:"create"})},{label:"还原备份",icon:"refresh",action:()=>window.__TAURI_INTERNALS__.invoke("request_layout_backup",{action:"manage"})},{label:"内存优化",icon:"refresh",action:()=>window.DeskBoxWidgets.optimizeMemory()},{label:"退出",icon:"power",danger:!0,action:()=>te.quit()}')
replace_once('l??=st(),te.watchOutsideClick(!0)}function X()', 'l??=st(),te.watchOutsideClick(!0),$t(()=>window.DeskBoxMotion.openBallMenu())}function X()')
replace_once('n.value&&(n.value=null,te.watchOutsideClick(!1)', 'n.value&&(window.DeskBoxMotion.closeBallMenu(),n.value=null,te.watchOutsideClick(!1)')
text+='\nwindow.DeskBoxMotion.bind({listen:Xt,label:as,state:B,hold:st,region:Ro});\n'
# Quickbar controls and geometry share published native work bounds.
replace_once('n.value==="behavior"?(L(),W("section",$f,[', 'n.value==="behavior"?(L(),W("section",$f,[d("div",{class:"card"},window.DeskBoxQuickbar.settings((tag,props,children)=>d(tag,props,children,-2),i.value),-2),')
replace_once('return{x:Math.round(Math.max(0,Math.min(window.innerWidth-F,Fe(y,le)))),y:Math.round(Math.max(0,Math.min(window.innerHeight-38,Fe(D,He))))}', 'return window.DeskBoxQuickbar.constrain(Math.round(Math.max(0,Math.min(window.innerWidth-F,Fe(y,le)))),Math.round(Math.max(0,Math.min(window.innerHeight-38,Fe(D,He)))),F,ne)')
start=text.index('function Se(x,y,D){',text.index('__name:"OverlayView"'));end=text.index('function K(x,y)',start)
text=text[:start]+'function Se(x,y,D){if(x.button!==0)return;ie.value=y,I.value&&G();window.DeskBoxQuickbar.itemPointer(x,y,D,{drag:H,target:K,commit:re})}'+text[end:]
text=text.replace('window.innerHeight-window.DeskBoxFenceMerge.geometry(i.value).y','window.DeskBoxQuickbar.bounds().bottom-window.DeskBoxFenceMerge.geometry(i.value).y')
text=text.replace('window.innerWidth-window.DeskBoxFenceMerge.geometry(i.value).x','window.DeskBoxQuickbar.bounds().right-window.DeskBoxFenceMerge.geometry(i.value).x')
replace_once('{label:"布局备份",icon:"folder",action:', 'window.DeskBoxQuickbar.menuEntry(B.config),{label:"布局备份",icon:"folder",action:')
text+='\nwindow.DeskBoxQuickbar.bind({state:B,label:Rn().label,listen:Xt,hold:st,region:Ro,focus:te.keyboardFocus,asset:An,invoke:(command,args)=>window.__TAURI_INTERNALS__.invoke(command,args)});\n'

# Plugin manager and widget menu integrate with the recovered Vue renderer.
replace_once('{id:"about",name:"关于",icon:"info"}', '{id:"plugins",name:"插件",icon:"puzzle"},{id:"about",name:"关于",icon:"info"}')
replace_once('n.value==="behavior"?(L(),W("section",$f,[', 'n.value==="plugins"?window.DeskBoxPlugins.settings((tag,props,children)=>d(tag,props,children,-2)):n.value==="behavior"?(L(),W("section",$f,[')
replace_once('...[["timer","倒计时与番茄钟","clock"]', '...window.DeskBoxPlugins.menu(a),...[["timer","倒计时与番茄钟","clock"]')
text+='\nwindow.DeskBoxPlugins.bind({state:B,listen:Xt,region:Ro,invoke:(command,args)=>window.__TAURI_INTERNALS__.invoke(command,args)});\n'

bundle.write_text(text, encoding='utf-8')
for name in ['rename-dialog.js', 'rename-dialog.css', 'menu-layout.js', 'menu-layout.css', 'fence-merge.js', 'fence-merge.css', 'media-background.js', 'media-background.css','widgets.js','widgets.css','widget-ball-motion.js','widget-ball-motion.css','cpu-temperature.js','sysmon-vram.js','quickbar-bridge.js','quickbar-bridge.css','plugins.js','plugins.css']:
    shutil.copy2(root / 'ui' / name, output / 'assets' / name)
html = (output / 'index.html').read_text(encoding='utf-8').replace('</head>',
    '<link rel="stylesheet" href="/assets/rename-dialog.css">\n'
    '<link rel="stylesheet" href="/assets/menu-layout.css">\n'
    '<script src="/assets/menu-layout.js"></script>\n'
    '<script src="/assets/rename-dialog.js"></script>\n'
    '<link rel="stylesheet" href="/assets/fence-merge.css">\n'
    '<script src="/assets/fence-merge.js"></script>\n'
    '<link rel="stylesheet" href="/assets/media-background.css">\n'
    '<script src="/assets/media-background.js"></script>\n'
    '<link rel="stylesheet" href="/assets/widgets.css">\n'
    '<script src="/assets/widgets.js"></script>\n'
    '<script src="/assets/plugins.js"></script>\n'
    '<link rel="stylesheet" href="/assets/plugins.css">\n'
    '<script src="/assets/cpu-temperature.js"></script>\n'
    '<script src="/assets/sysmon-vram.js"></script>\n'
    '<script src="/assets/quickbar-bridge.js"></script>\n'
    '<link rel="stylesheet" href="/assets/quickbar-bridge.css">\n'
    '<link rel="stylesheet" href="/assets/widget-ball-motion.css">\n'
    '<script src="/assets/widget-ball-motion.js"></script>\n</head>')
(output / 'index.html').write_text(html, encoding='utf-8')
for name in ['quickbar.html','quickbar.js','quickbar.css']:
    shutil.copy2(root / 'ui' / name, output / name)
records = [{'path': p.relative_to(output).as_posix(), 'sha256': hashlib.sha256(p.read_bytes()).hexdigest()}
           for p in sorted(output.rglob('*')) if p.is_file()]
(root / 'evidence/modified-assets.json').write_text(json.dumps(records, indent=2), encoding='utf-8')
print('Patched frontend: recovered menu placement + rename/focus bridge')

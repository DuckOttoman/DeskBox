const assert = require('node:assert/strict');
const fs = require('node:fs'); const vm = require('node:vm');
const window = {}; vm.runInNewContext(fs.readFileSync('ui/menu-layout.js','utf8'),{window});
const place = (anchor,size,viewport) => JSON.parse(JSON.stringify(window.DeskBoxMenuLayout.place(anchor,size,viewport)));
let r = place({x:100,y:699,h:'right',v:'down'},{width:220,height:650},{width:1920,height:1080});
assert.equal(r.top,'422px'); assert.equal(r.left,'100px');
r = place({x:1918,y:1078,h:'right',v:'down'},{width:260,height:260},{width:1920,height:1080});
assert.equal(r.left,'1652px'); assert.equal(r.top,'812px');
r = place({x:50,y:40,h:'left',v:'up'},{width:240,height:600},{width:1920,height:1080});
assert.equal(r.left,'8px'); assert.equal(r.top,'8px');
r = place({x:800,y:700,h:'left',v:'up'},{width:240,height:200},{width:1920,height:1080});
assert.equal(r.left,'560px'); assert.equal(r.top,'500px');
r = place({x:900,y:100,h:'right',v:'down'},{width:260,height:3000},{width:1920,height:1080});
assert.equal(r.top,'8px');
for (const width of [280,640,1920]) for (const height of [240,720,1080]) {
 for (const x of [0,width/2,width]) for (const y of [0,height/2,height]) {
  r = place({x,y,h:'right',v:'down'},{width:220,height:height-16},{width,height});
  const left=parseFloat(r.left), top=parseFloat(r.top);
  assert(left>=8 && left+Math.min(220,width-16)<=width-8);
  assert(top>=8 && top+height-16<=height-8);
 }
}
console.log('MENU_LAYOUT_OK: expanded/bottom/right/up/narrow/oversized viewport placement');

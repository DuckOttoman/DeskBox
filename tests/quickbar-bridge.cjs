const assert=require('node:assert/strict'),{clamp}=require('../ui/quickbar-bridge.js');
assert.deepEqual(clamp([500,900],[320,260],[0,0,1000,704]),{x:500,y:444});
assert.deepEqual(clamp([-20,-10],[320,260],[12,12,976,680]),{x:12,y:12});
console.log('quickbar preview geometry passed');

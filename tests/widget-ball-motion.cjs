const assert=require('node:assert/strict');
const {duration}=require('../ui/widget-ball-motion.js');
assert.equal(duration(240,1,false),240);
assert.equal(duration(240,2,false),120);
assert.equal(duration(240,0,false),0);
assert.equal(duration(240,1,true),0);
assert.equal(duration(240,NaN,false),240);
assert.equal(duration(240,.01,false),1200);
console.log('MOTION_POLICY_TESTS_PASSED');

const assert=require('node:assert/strict');
const {form,remaining,nextPhoto,weatherText}=require('../ui/widgets.js');
assert.equal(form(220,150),'small');assert.equal(form(320,240),'medium');assert.equal(form(460,320),'large');assert.equal(form(460,100),'small');
assert.equal(remaining({running:true,deadline:11000},6000),5000);
assert.equal(remaining({running:true,deadline:11000},16000),0);
assert.equal(remaining({running:false,remaining:9000},16000),9000);
assert.equal(nextPhoto(2,3),0);assert.equal(nextPhoto(0,0),0);
assert.equal(weatherText(0),'晴');assert.equal(weatherText(61),'雨');
// The recovered compiler helper requires pre-normalized string class names.
const vm=require('node:vm'),fs=require('node:fs');
const context={window:{addEventListener(){}},document:{visibilityState:'visible'},console,Date,Map,Set};
vm.runInNewContext(fs.readFileSync(require.resolve('../ui/widgets.js'),'utf8'),context);
const api=context.window.DeskBoxWidgets;
const widget={id:'test',kind:'notes',style:{widgetWidth:320,widgetHeight:240,widgetData:{note:'saved',todos:[{id:'todo',text:'task',done:true}]}}};
api.bind({state:{config:{widgets:[widget]}},asset:x=>x});
const tree=api.content((tag,props,children)=>({tag,props,children}),widget);
function walk(n){if(!n||typeof n!=='object')return;if(n.props?.class)assert.equal(typeof n.props.class,'string');for(const child of Array.isArray(n.children)?n.children:[])walk(child);}
walk(tree);assert.equal(tree.children[2].children[0].props.class,'db-todo done');
console.log('WIDGET_LOGIC_OK');

const {weatherEmoji,ipCity}=require("../ui/widgets.js");
assert.equal(weatherEmoji(0),"☀️");assert.equal(weatherEmoji(95),"⛈️");assert.equal(weatherEmoji(73),"❄️");assert.equal(weatherEmoji(null),"🌡️");assert.equal(ipCity({city:"Shanghai",latitude:31,longitude:121}).name,"Shanghai");assert.equal(ipCity({city:"bad",latitude:999,longitude:121}),null);

assert.equal(ipCity({city:"Test",loc:"31.2,121.4"},"ipinfo.io").latitude,31.2);assert.equal(ipCity({city:"Bad",loc:","},"ipinfo.io"),null);

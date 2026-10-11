const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');
class Element {
  constructor(tag) { this.tag = tag; this.children = []; this.events = {}; this.dataset = {}; this.value = ''; }
  append(...children) { for (const c of children) { c.parent = this; this.children.push(c); } }
  setAttribute(key, value) { this[key] = value; }
  addEventListener(type, callback) { this.events[type] = callback; }
  focus() { document.activeElement = this; }
  setSelectionRange(start, end) { this.selection = [start, end]; }
  remove() { this.parent.children = this.parent.children.filter(c => c !== this); }
  find(id) { return this.id === id ? this : this.children.map(c => c.find(id)).find(Boolean); }
  dispatch(type, extra = {}) { return this.events[type]?.({preventDefault(){}, stopPropagation(){}, ...extra}); }
}
global.document = {body: new Element('body'), createElement: tag => new Element(tag), getElementById: id => document.body.find(id)};
const calls = []; let fail = false; let held = 0; let pending = false; let complete;
global.window = {__TAURI_INTERNALS__: {invoke: async (command, args) => { calls.push([command,args]); if (fail) throw '同名文件已存在'; if (pending) await new Promise(resolve => complete = resolve); }}};
vm.runInThisContext(fs.readFileSync('ui/rename-dialog.js', 'utf8'));
(async () => {
  window.DeskBoxRename.bind({focus: async on => calls.push(['focus',on]), hold: () => {held++; return () => held--;}, region(){}});
  await window.DeskBoxRename.open({id:'one', name:'中文报告.txt', kind:'file'});
  const input = document.getElementById('deskbox-rename-input');
  assert(input, 'rename action must open a filename input');
  assert.equal(input.value, '中文报告.txt'); assert.deepEqual(input.selection, [0,4]); assert.equal(held,1);
  const confirm = document.getElementById('deskbox-rename-confirm'); confirm.focus();
  await document.getElementById('deskbox-rename-dialog').dispatch('keydown',{key:'Tab'});
  assert.equal(document.activeElement, input, 'Tab must wrap inside the rename dialog');
  await document.getElementById('deskbox-rename-dialog').dispatch('keydown',{key:'Tab', shiftKey:true});
  assert.equal(document.activeElement, confirm, 'Shift+Tab must wrap inside the rename dialog');
  input.value = '新报告.txt'; fail = true;
  await document.getElementById('deskbox-rename-form').dispatch('submit');
  assert.equal(document.getElementById('deskbox-rename-error').textContent, '同名文件已存在');
  assert(document.getElementById('deskbox-rename-dialog'), 'error keeps dialog open');
  fail = false;
  await document.getElementById('deskbox-rename-form').dispatch('submit');
  assert(calls.some(([c,a]) => c === 'rename_item' && a.id === 'one' && a.name === '新报告.txt'));
  assert.equal(document.getElementById('deskbox-rename-dialog'), undefined); assert.equal(held,0);
  await window.DeskBoxRename.open({id:'two',name:'folder',kind:'folder'});
  await document.getElementById('deskbox-rename-dialog').dispatch('keydown',{key:'Escape'});
  assert.equal(held,0); assert.equal(document.getElementById('deskbox-rename-dialog'), undefined);
  await window.DeskBoxRename.open({id:'sys',name:'回收站',kind:'system'});
  assert.equal(document.getElementById('deskbox-rename-dialog'), undefined);
  await window.DeskBoxRename.open({id:'three',name:'pending.txt',kind:'file'});
  pending = true;
  const submission = document.getElementById('deskbox-rename-form').dispatch('submit');
  const busyInput = document.getElementById('deskbox-rename-input');
  assert.equal(busyInput.readOnly, true); assert(!busyInput.disabled); assert.equal(document.activeElement, busyInput);
  await document.getElementById('deskbox-rename-dialog').dispatch('keydown',{key:'Tab'});
  assert.equal(document.activeElement, busyInput, 'busy dialog retains focus inside');
  complete(); await submission;
  assert.equal(held,0);
  console.log('RENAME_UI_OK: prefill/selection, error/retry, invoke payload, cancel, system exclusion, Tab trap and busy focus');
})().catch(error => {console.error(error); process.exitCode=1;});

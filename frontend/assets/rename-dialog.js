(() => {
  let bridge;
  const element = (tag, id, text) => {
    const node = document.createElement(tag);
    if (id) node.id = id;
    if (text) node.textContent = text;
    return node;
  };
  window.DeskBoxRename = {
    bind(api) { bridge = api; },
    async open(item) {
      if (item.kind === 'system' || document.getElementById('deskbox-rename-dialog')) return;
      const backdrop = element('div', 'deskbox-rename-dialog');
      backdrop.className = 'deskbox-rename-backdrop';
      backdrop.setAttribute('role', 'dialog');
      backdrop.setAttribute('aria-modal', 'true');
      backdrop.setAttribute('aria-labelledby', 'deskbox-rename-title');
      backdrop.dataset.hit = '';
      const form = element('form', 'deskbox-rename-form');
      const title = element('h3', 'deskbox-rename-title', '重命名');
      const label = element('label', null, '新名称');
      label.htmlFor = 'deskbox-rename-input';
      const input = element('input', 'deskbox-rename-input');
      input.type = 'text'; input.value = item.name; input.autocomplete = 'off'; input.spellcheck = false;
      const note = element('p', null, item.kind === 'shortcut' ? '快捷方式后缀会自动保留。' : '文件扩展名可编辑；改变扩展名可能影响打开方式。');
      note.className = 'deskbox-rename-note';
      const error = element('p', 'deskbox-rename-error'); error.setAttribute('role', 'alert');
      const actions = element('div'); actions.className = 'deskbox-rename-actions';
      const cancel = element('button', 'deskbox-rename-cancel', '取消'); cancel.type = 'button';
      const confirm = element('button', 'deskbox-rename-confirm', '确认'); confirm.type = 'submit';
      actions.append(cancel, confirm);
      form.append(title, label, input, note, error, actions); backdrop.append(form);
      document.body.append(backdrop);
      const release = bridge.hold();
      let busy = false;
      const close = async () => {
        if (busy) return;
        backdrop.remove(); release(); bridge.region();
        try { await bridge.focus(false); } catch (_) { /* Window may be closing. */ }
      };
      cancel.addEventListener('click', close);
      backdrop.addEventListener('pointerdown', event => event.stopPropagation());
      backdrop.addEventListener('click', event => event.stopPropagation());
      backdrop.addEventListener('keydown', async event => {
        event.stopPropagation();
        if (event.key === 'Escape') { event.preventDefault(); await close(); }
        if (event.key === 'Tab') {
          const controls = [input, cancel, confirm].filter(node => !node.disabled);
          const index = controls.indexOf(document.activeElement);
          if (!controls.length || index < 0 || (!event.shiftKey && index === controls.length - 1) || (event.shiftKey && index === 0)) {
            event.preventDefault();
            (event.shiftKey ? controls[controls.length - 1] : controls[0])?.focus();
          }
        }
      });
      form.addEventListener('submit', async event => {
        event.preventDefault(); if (busy) return;
        busy = true; confirm.disabled = true; cancel.disabled = true; input.readOnly = true; input.focus(); error.textContent = '';
        try {
          await window.__TAURI_INTERNALS__.invoke('rename_item', {id: item.id, name: input.value});
          busy = false; await close();
        } catch (failure) {
          error.textContent = String(failure);
          busy = false; confirm.disabled = false; cancel.disabled = false; input.readOnly = false; input.focus();
        }
      });
      try {
        await bridge.focus(true); input.focus();
        const dot = item.kind === 'file' ? input.value.lastIndexOf('.') : -1;
        input.setSelectionRange(0, dot > 0 ? dot : input.value.length);
      } catch (failure) { error.textContent = `无法获取键盘焦点：${failure}`; }
    }
  };
})();

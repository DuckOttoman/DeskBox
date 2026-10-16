/* DeskBox injects the SDK before this entry document executes. */
(async () => {
  const sdk = window.DeskBox;
  let settings = await sdk.getSettings();
  let timer;
  function render() {
    const now = new Date();
    document.querySelector('#clock').textContent = now.toLocaleTimeString([], {hour:'2-digit', minute:'2-digit', hour12:!!settings.hour12});
    document.querySelector('#date').textContent = now.toLocaleDateString([], {weekday:'short', month:'short', day:'numeric'});
  }
  function visible(value) {
    clearInterval(timer);
    if (value) { render(); timer = setInterval(render,1000); }
  }
  function resize(size) {
    document.body.dataset.form = size.form;
    document.querySelector('#state').textContent = `${size.width} × ${size.height} · ${size.form}`;
  }
  function theme(value) {
    document.documentElement.style.colorScheme = value.mode === 'light' ? 'light' : 'dark';
  }
  sdk.onResize(resize);
  sdk.onThemeChange(theme);
  sdk.onVisibilityChange(visible);
  document.querySelector('#format').onclick = async () => {
    settings = {...settings, hour12:!settings.hour12};
    await sdk.setSettings(settings); render();
  };
  const initial = await sdk.ready();
  theme(initial.theme); resize(initial.size); visible(initial.visible);
})().catch(error => { document.body.textContent = `Clock: ${error.message}`; });

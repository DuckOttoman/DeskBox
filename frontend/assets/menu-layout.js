// Keep the recovered inline menus inside the current WebView viewport.
(() => {
  window.DeskBoxMenuLayout = {
    place(anchor, size, viewport) {
      const margin = 8;
      const width = Math.min(size.width, Math.max(0, viewport.width - margin * 2));
      const height = Math.min(size.height, Math.max(0, viewport.height - margin * 2));
      const clamp = (value, max) => Math.max(margin, Math.min(value, Math.max(margin, max)));
      const left = clamp(anchor.h === 'left' ? anchor.x - width : anchor.x, viewport.width - width - margin);
      const top = clamp(anchor.v === 'up' ? anchor.y - height : anchor.y, viewport.height - height - margin);
      return {left: `${left}px`, top: `${top}px`,
        transformOrigin: `${anchor.v === 'up' ? 'bottom' : 'top'} ${anchor.h === 'left' ? 'right' : 'left'}`};
    }
  };
})();

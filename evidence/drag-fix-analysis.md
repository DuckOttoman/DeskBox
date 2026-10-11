# 文件夹框拖动回弹定位

用户症状：左键拖动标题能移动一下，然后恢复原位。

定位方法：在 `--drag-session` 隔离诊断模式记录 pointerdown/move/up/cancel、lostpointercapture、blur/focus。先以跳过原生焦点的窗口对照，再令诊断执行与运行版相同的 keyboard_focus。测试配置与用户配置分离。

根因证据：`drag-fix-focus-repro/runtime.log` 的真实鼠标事件顺序为 pointerdown → pointermove（dragging=true）→ blur（dragging=true）→ lostpointercapture（dragging=false）→ pointerup（dragging=false）。原生 SetFocus(host) 再 WebView2.MoveFocus 会暂时使 WebView 失焦；ui/fence-merge.js 将自身的焦点交接当作用户取消，调用 finish(null) 恢复 original 坐标。先前隐藏 WebView 测试跳过原生焦点调用，因此漏测。

修复：仅在当前拖动正在等待 keyboard_focus(true) 交接完成时忽略 blur。完成后真实 blur 继续取消；pointercancel、Esc、匹配左键松开提交规则保留。不使用固定延迟，也不去掉失焦保护。

回归：在指针状态机测试中模拟原生交接的异步 blur，断言移动后不回弹、左键松开保存，交接完成后的真实 blur 仍取消。先观察断言失败（drag-fix-pointer-red.log），再验证修复通过（drag-fix-pointer-green.log）。真实鼠标修复复测及发布校验见 drag-fix-verification.json。

边界：真实指针复测为独立可见诊断窗口，使用真实 Win32/WebView2 焦点流程。桌面子窗口无法由当前窗口工具枚举并直接控制；完整五秒真实长按、多屏 DPI 和逐主题动画未实测。

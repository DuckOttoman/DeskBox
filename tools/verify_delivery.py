"""Check artifact provenance, command coverage and recorded runtime result."""
import hashlib
import json
from pathlib import Path
import re
import struct

root = Path(__file__).resolve().parents[1]
digest = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
assets = json.loads((root / 'evidence/assets.json').read_text(encoding='utf-8'))
for asset in assets:
    assert digest(root / 'dist' / asset['path'].lstrip('/')) == asset['sha256'], asset['path']
modified = json.loads((root / 'evidence/modified-assets.json').read_text(encoding='utf-8'))
for asset in modified:
    assert digest(root / 'frontend' / asset['path']) == asset['sha256'], asset['path']
rules = json.loads((root / 'evidence/classification-rules.json').read_text(encoding='utf-8'))
commands = json.loads((root / 'evidence/commands.json').read_text(encoding='utf-8'))
app = (root / 'src/app.rs').read_text(encoding='utf-8')
registered = set(re.findall(r'commands::(\w+)', app))
assert set(commands) <= registered
assert 'rename_item' in registered
build = (root / 'evidence/animation-fix-build.log').read_text(encoding='utf-8')
assert 'Finished `release` profile' in build
tests = (root / 'evidence/fence-merge-core.log').read_text(encoding='utf-8')
assert '14 passed; 0 failed' in tests
merge_runtime = (root / 'evidence/animation-fix-green/runtime.log').read_text(encoding='utf-8')
assert 'SMOKE_RESULT success=true' in merge_runtime and 'FRONTEND_ERROR' not in merge_runtime
for check in ['held_contents_never_commits', 'cancel_restores_all', 'contents_release_commits', 'restore_consumed_and_split', 'group_drag_detaches_current']:
    assert check in merge_runtime, check
for name in ['animation-fix-menu13', 'animation-fix-menu60', 'animation-fix-settings']:
    fresh = (root / 'evidence' / name / 'runtime.log').read_text(encoding='utf-8')
    assert 'SMOKE_RESULT success=true' in fresh and 'FRONTEND_ERROR' not in fresh, name
assert '8 passed; 0 failed' in tests
assert '7 passed; 0 failed' in tests
assert '5 passed; 0 failed' in tests
assert '4 passed; 0 failed' in tests and '2 passed; 0 failed' in tests
latest = (root / 'evidence/learning-native/runtime.log').read_text(encoding='utf-8')
assert 'SMOKE_RESULT success=true' in latest and 'FRONTEND_ERROR' not in latest
for check in ['orphan_fence_ipc_save','orphan_widget_ipc_save','native_lnk_target_learning','native_url_learning','native_move_keeps_scan_category','native_alias_shared_learning','native_reset_learning']:
    assert check in latest, check
latest60=(root/'evidence/learning-menu60/runtime.log').read_text(encoding='utf-8')
assert 'SMOKE_RESULT success=true' in latest60 and 'FRONTEND_ERROR' not in latest60
runtime = (root / 'evidence/settings-regression-13/runtime.log').read_text(encoding='utf-8')
assert 'SMOKE_RESULT success=true' in runtime
assert 'FRONTEND_CONTRACT_CHECKS' in runtime and 'FRONTEND_ERROR' not in runtime
assert 'rename_menu' in runtime and 'rename_item' in runtime
for name in ['original_menu_entries', 'move_submenu_expands', 'move_destination_labels', 'move_leaf_dispatch_and_close', 'move_last_row_reachable', 'bottom_right_menu_bounds', 'delete_confirmation_submenu', 'delete_parent_does_not_execute']:
    assert name in runtime, name
long_menu = (root / 'evidence/settings-regression-60/runtime.log').read_text(encoding='utf-8')
assert 'SMOKE_RESULT success=true' in long_menu and 'FRONTEND_ERROR' not in long_menu
for log in [runtime,long_menu]:
    assert 'KEYBOARD_NATIVE_FOCUS label=overlay-0 owned=true' in log
    for check in ['fence_own_menu_entries','fence_delete_confirmation','fence_collapse_expand','fence_style_panel_persists','fence_search_filters','fence_title_rename','widget_own_menu_entries','widget_opacity_choices','widget_opacity_persists_and_checkmark','widget_remove_leaf']:
        assert check in log, check
assert 'MENU_LAYOUT_OK' in (root / 'evidence/menu-layout-green.log').read_text(encoding='utf-8')
assert (root / 'evidence/settings-regression-13/rename-fixtures/rename-fixture.txt').read_text() == 'DeskBox rename fixture content'
assert 'RENAME_UI_OK' in (root / 'evidence/rename-ui-green.log').read_text(encoding='utf-8')
for name in ['settings-first-open-final','tray-settings-final']:
    lifecycle=(root/'evidence'/name/'runtime.log').read_text(encoding='utf-8')
    assert 'SMOKE_RESULT success=true' in lifecycle and 'FRONTEND_ERROR' not in lifecycle
    assert lifecycle.count('SETTINGS_CREATED') == 1
    for check in ['ball_settings_first_open','settings_concurrent_first_open','ball_settings_reopen','ball_settings_restore_minimized','ball_settings_repeated_open']:
        assert check in lifecycle, check
scan=(root/'evidence/scan-responsiveness-final/runtime.log').read_text(encoding='utf-8')
assert 'SMOKE_RESULT success=true' in scan and 'FRONTEND_ERROR' not in scan
for check in ['scan_preserves_concurrent_edit','scan_rename_serialization','scan_responsiveness']:
    assert check in scan, check
native = json.loads((root / 'evidence/native-fidelity-final/native-check.json').read_text(encoding='utf-8'))
assert native['initial_visible'] and not native['hidden_visible'] and native['restored_visible']
assert native['owned_during_hide'] and native['visibility_restored']
assert native['items'] == native['icons'] and native['icons'] > 0
assert native['original_layout_items_matched'] == 195
comparison = json.loads((root / 'evidence/native-fidelity-pixel-comparison.json').read_text(encoding='utf-8'))
assert comparison['pixel_identical_to_original_cache'] >= 60
assert comparison['final_comparable_preliminary_matches'] == 178
assert comparison['final_mean_absolute_channel_error_max'] < 1.0
exe = root / 'bin/deskbox-reconstructed.exe'
loader = root / 'bin/WebView2Loader.dll'
assert loader.exists() and digest(loader) == digest(root / 'target/release/WebView2Loader.dll')
binary = exe.read_bytes()
pe = struct.unpack_from('<I', binary, 60)[0]
optional = struct.unpack_from('<H', binary, pe + 20)[0]
sections = [struct.unpack_from('<8sIIII', binary, pe + 24 + optional + i * 40)
            for i in range(struct.unpack_from('<H', binary, pe + 6)[0])]
def offset(rva):
    for _, _, virtual, size, raw in sections:
        if virtual <= rva < virtual + size:
            return raw + rva - virtual
    raise ValueError(hex(rva))
imports_rva = struct.unpack_from('<I', binary, pe + 24 + 112 + 8)[0]
imports = []
for cursor in range(offset(imports_rva), len(binary), 20):
    fields = struct.unpack_from('<IIIII', binary, cursor)
    if not any(fields):
        break
    pos = offset(fields[3])
    imports.append(binary[pos:binary.index(b'\0', pos)].decode('ascii'))
original = root.parent / 'deskbox.exe'
expected = '86712e4115ffe403c153bb96803c755a5afe6efa9f8f02171f4c62326f3509a7'
assert digest(original) == expected
result = dict(original_sha256=expected, original_unchanged=True, recovered_assets_verified=len(assets),
              categories=len(rules['categories']), extensions=sum(map(len, rules['extensions'].values())),
              keywords=sum(map(len, rules['keywords'].values())), original_commands_registered=len(commands),
              core_tests_passed=40, fence_merge_native_checks_passed=True, real_pointer_full_merge_flow_verified=False, learning_and_orphan_monitor_native_checks_passed=True, rename_filesystem_tests_passed=7, rename_context_menu_webview_test_passed=True,
              modified_assets_verified=len(modified), release_build_passed=True, release_webview_smoke_passed=True,
              executable_bytes=exe.stat().st_size, executable_sha256=digest(exe), webview2_loader_sha256=digest(loader), imports=imports,
              menu_13_fences_webview_passed=True, menu_60_fences_scroll_webview_passed=True, native_desktop_hide_restore_passed=True, original_layout_ids_matched=native['original_layout_items_matched'],
              original_cache_pixel_matches=comparison['pixel_identical_to_original_cache'], added_commands=['rename_item'], desktop_attached_own_menus_passed=True, native_edit_focus_passed=True, settings_first_open_lifecycle_passed=True, tray_settings_shared_path_passed=True, slow_scan_and_concurrent_rename_passed=True, review='See settings-window-review.md and fence-widget-interaction-review.md.',
              fence_merge_runtime_checks=[line for line in merge_runtime.splitlines() if line.startswith('FRONTEND_CONTRACT_CHECKS')],
              historical_checks=['native desktop visibility, pixel fidelity, learning and scan checks use their recorded earlier builds'],
              runtime_checks=[line for line in runtime.splitlines() if line.startswith('FRONTEND_CONTRACT_CHECKS')],
              learning_runtime_checks=[line for line in latest.splitlines() if line.startswith('FRONTEND_CONTRACT_CHECKS')],
              limitations=['Original Rust source is not recovered verbatim.', 'Physical drag/resize, hotplug, autostart and recycle actions lack end-to-end tests.',
                           'Unsupported temperatures/GPU metrics are null.'])
(root / 'evidence/verification.json').write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding='utf-8')
print(json.dumps(result, ensure_ascii=False, indent=2))

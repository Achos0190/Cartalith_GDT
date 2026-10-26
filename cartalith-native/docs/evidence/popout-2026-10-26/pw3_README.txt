PW-3 evidence (2026-10-26, one monitor, base b75eef7d). Section 11 of POPOUT_WINDOWS_SCOPE.md reads these.

pw3_windows_probe_desktop.txt / _tablet.txt / _phone.txt   final runs of _popout_windows_probe.gd (457 / 21 / 21 checks, 0 failed, 0 SCRIPT ERROR); the desktop log includes the real LM Studio qwen3-14b section
pw3_identity_compare.txt                 _popout_windows_id_probe.gd on the changed tree: RESULT IDENTICAL, 0 pixels outside the button (baselines taken twice on the pristine tree)
pw3_identity_compare_first_run_before_rules_fix.txt   the same probe BEFORE the generation-rules header fix: 11 601 pixels outside the button in t_rules
pw3_identity_pristine_rules.png / pw3_identity_changed_rules.png   the generation-rules window on both trees (the changed image is the post-fix capture; the pre-fix one was not kept)
pw3_mutation.txt                         16 mutants, 16 killed (first block: 9 applied; second block: the 7 rows whose pattern missed CRLF files, re-run)
pw3_regression.txt                       the existing probes re-run with the changed tree
pw3_window_*.png                         each window as a native window's own texture (the OS title bar is not part of it)
pw3_assistant_native_streaming / _reply / _card .png   the assistant native, a partial reply, the finished reply, a real model's proposal card
pw3_busy_assistant.png / pw3_busy_place_editor.png   the card mirror over two muted native windows during a generate
pw3_opener_place_editor_native.png       the place editor opened from the native roster

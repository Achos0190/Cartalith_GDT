PW-2 evidence (2026-10-26, one monitor, base 205252ed). Section 10 of POPOUT_WINDOWS_SCOPE.md reads these.

pw2_roster_probe_desktop.txt / _tablet.txt / _phone.txt  final runs of _popout_roster_probe.gd (146 / 6 / 6 checks, 0 failed, 0 SCRIPT ERROR); the desktop log includes the _diag popup lines and the hashes
pw2_identity_baseline_pristine.txt   _popout_roster_id_probe.gd --out on the pristine tree (run twice for the two baselines)
pw2_identity_compare.txt             the same file on the changed tree: RESULT: IDENTICAL, 0 pixels outside the button, 0 with it hidden
pw2_identity_pristine_economy.png / pw2_identity_changed_economy.png   the Economy tab on both trees (880 x 620 crop); the only difference is the Pop out button at the lower left
pw2_regression.txt                   one line per regression probe run (setting off), and the four roster-touching probes that fail identically on the pristine tree
pw2_mutation.txt                     the mutation harness output (27 of 29 killed by the roster probe, 3 of 3 by the identity probe; two stated survivors)
pw2_roster_native.png                the roster as a native window (the window's own texture; the OS title bar is not part of it)
pw2_roster_docked.png                the main window with the roster docked and the Pop out button visible
pw2_roster_native_during_generate.png   the card mirror over the muted native window during a generate
pw2_roster_native_scale_1_5_simulated.png   fit_scale with a simulated second-monitor scale of 1.5: 1320 x 930 window, 880 x 620 logical
pw2_roster_native_colour_popup.png / pw2_roster_native_transfer_menu.png   popups of a native window (embedded in it, clipped to it)

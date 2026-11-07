# Talos TUI

## Native key repeat

On terminals that report native Press/Repeat/Release events (including the Kitty keyboard
protocol used by Ghostty), hold Backspace/Delete or navigation keys to repeat their existing
editing, picker or viewport action. Timing comes from the terminal/OS; Talos has no repeat timer.
Composer deletion and Left/Right movement use Unicode grapheme boundaries, including combining
marks and joined emoji. Delete removes the grapheme at the cursor.

Enter, approval digits, confirmations, Escape and other action shortcuts do not execute on Repeat.
A repeat sequence is bound to its original panel; panel replacement, focus change, paste, resize
or session change requires a fresh Press before repetition resumes. This deliberately stops a
held key at a focus boundary. Terminals that report only Press events cannot distinguish physical
presses from OS repeat; Talos does not invent timing heuristics to infer that distinction.

Protocol regression: `python3 scripts/accept_native_key_repeat.py /path/to/talos` runs the real
TUI in a disposable PTY against a loopback provider. This is protocol/execution evidence, not a
physical keyboard or IME certification. I318 tracks Ghostty (no tmux) and the second-terminal
hold/release/IME acceptance separately.

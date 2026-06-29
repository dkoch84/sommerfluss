# sommerfluss

A manual, frame-tree tiling **window manager** for [river](https://codeberg.org/river/river)
0.4+ — a herbstluftwm successor on Wayland, with **virtual / overlapping monitors**
as a first-class, non-negotiable design constraint.

Two binaries, mirroring herbstluftwm:

| sommerfluss | herbstluftwm | role |
|---|---|---|
| **`sfwm`** | `herbstluftwm` | the window manager (a `river-window-management-v1` client) |
| **`sc`**   | `herbstclient` | the control client; config is a bash script calling `sc` |

## Status — milestone 2 (monitors)

Done:

- **Milestone 1 (skeleton):** connects to river, tracks outputs/windows/seats,
  drives the manage/render sequence loop correctly. (Carried over and corrected
  from the `hlwl` skeleton — notably `show`/`hide` are *rendering* state and now
  run in the render pass, not the manage pass.)
- **Milestone 2 (monitors) — the project's defining requirement:**
  - A WM-side `Monitor` model (`sfwm/src/monitor.rs`), decoupled from river outputs:
    arbitrary logical rects, stacking `z`, `pad`, displayed `tag`, `locked_tag`.
  - `set_monitors` / `add_monitor` / `raise_monitor` / `lock_tag` / `pad`, plus
    `focus_monitor` / `cycle_monitor` and a `detect_monitors` fallback.
  - **Overlapping overlay monitors** (the hlwm `float1`/`float2` trick): an overlay
    on the same rect as a base monitor, raised, renders *above* it. The global
    render list is ordered by `(monitor.z, intra-monitor stack)` using
    `river_node_v1` `place_bottom`/`place_above`.
  - Driven at runtime over the `sc` IPC socket — a near-direct port of the hlwm
    autostart's monitor section (see `sfwm/examples/autostart`).
  - Pure-logic core is unit-tested (`cargo test -p sfwm`), including the
    overlapping-overlay requirement, without needing a running river.

Placeholder until later milestones: the per-monitor layout is a "monocle" (the
topmost window of a monitor's tag fills its usable rect; others hidden). The frame
tree + full tag model (with `my_monitor` affinity) is milestone 3; the attribute
tree, keybinds, rules, theming, and scratchpad save/load follow.

Deferred and noted in code: resolving a river output's numeric `wl_output` global
name to a human name like `DP-1` (needs binding the `wl_output` global). The
monitor model only needs output *geometry*, which is fully handled.

## Build

```sh
cargo build --release        # builds both sfwm and sc
cargo test -p sfwm           # runs the monitor-model unit tests
```

`wayland-client` dlopens `libwayland-client` at runtime, so it must be present to
run (but is not a link-time dependency).

## Run

river advertises `river_window_manager_v1` only to its designated window manager,
so `sfwm` must be launched *by* river.

1. Install river's init (execs `sfwm`):
   ```sh
   install -m755 sfwm/examples/river-init ~/.config/river/init
   ```
2. Install the sommerfluss autostart (the bash config `sfwm` runs; calls `sc`):
   ```sh
   install -Dm755 sfwm/examples/autostart ~/.config/sommerfluss/autostart
   ```
3. Put `sfwm` and `sc` on `PATH`, then start river from a TTY:
   ```sh
   river
   ```

Because river isolates the WM in a separate process, an `sfwm` crash does not kill
the session — you can iterate and even hot-swap window managers live.

### IPC socket

`sfwm` listens on `$SOMMERFLUSS_SOCKET`, defaulting to
`$XDG_RUNTIME_DIR/sfwm-$WAYLAND_DISPLAY.sock`. `sc` resolves the same path. Try:

```sh
sc list_monitors
sc add_monitor 3840x2160+1440+0 8 float1 && sc raise_monitor float1 && sc lock_tag 8 float1
sc list_outputs
```

## Layout

```
sfwm/                  the window manager
  src/main.rs          connection, event loop (calloop), manage/render passes
  src/monitor.rs       the Monitor model + overlapping-monitor logic (+ unit tests)
  src/ipc.rs           the sc command dispatcher
  src/protocol.rs      river-window-management-v1 bindings (from the vendored XML)
  protocols/           vendored protocol XML (re-vendor when bumping river)
  examples/            river-init and the sommerfluss autostart
sc/                    the control client (herbstclient successor)
```

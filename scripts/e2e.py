#!/usr/bin/env python3
"""Dual-build example E2E driver.

Runs a TUI example through a scripted input sequence and records the
reconstructed terminal screen (cells + SGR attributes) after every phase,
plus the exit status. The same spec runs against the upstream Go binary and
the Rust port; the runner then requires the Rust screens to be 1:1 identical
to the Go screens and both to satisfy the spec's `expect` fragments.

Spec format (JSON):
{
  "width": 80, "height": 24,
  "phases": [
    {"keys": "q", "settle": 0.6, "expect": ["fragment"], "expect_not": ["x"]},
    {"resize": [100, 30], "settle": 0.6, "expect": [...]},
    ...
  ],
  "expect_exit": true
}

`keys` uses the same escapes as pty_driver.py (unicode_escape) plus:
  phases separated by "|" are sent `gap` seconds apart.
Mouse events and escape sequences are passed through literally
(e.g. "\\x1b[<0;40;12M" is an SGR left-click at 40,12).
"""
import argparse
import fcntl
import json
import os
import pty
import select
import struct
import sys
import termios
import time

US = 0x1F  # unit separator


def set_win_size(fd, width, height):
    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", height, width, 0, 0))


def strip_sgr(seq):
    # Parse an SGR sequence into (fg, bg, bold, reverse).
    params = seq
    if not params:
        params = "0"
    fg, bg, bold, reverse = None, None, False, False
    vals = [int(v) if v else 0 for v in params.split(";")]
    i = 0
    while i < len(vals):
        v = vals[i]
        if v == 0:
            fg, bg, bold, reverse = None, None, False, False
        elif v == 1:
            bold = True
        elif v == 7:
            reverse = True
        elif 30 <= v <= 37:
            fg = ("basic", v - 30)
        elif v == 38:
            if i + 2 < len(vals) and vals[i + 1] == 5:
                fg = ("idx", vals[i + 2]); i += 2
            elif i + 4 < len(vals) and vals[i + 1] == 2:
                fg = ("rgb", vals[i + 2], vals[i + 3], vals[i + 4]); i += 4
        elif v == 39:
            fg = None
        elif 40 <= v <= 47:
            bg = ("basic", v - 40)
        elif v == 48:
            if i + 2 < len(vals) and vals[i + 1] == 5:
                bg = ("idx", vals[i + 2]); i += 2
            elif i + 4 < len(vals) and vals[i + 1] == 2:
                bg = ("rgb", vals[i + 2], vals[i + 3], vals[i + 4]); i += 4
        elif v == 49:
            bg = None
        i += 1
    return (fg, bg, bold, reverse)


class Screen:
    def __init__(self, rows, cols):
        self.rows, self.cols = rows, cols
        self.grid = {}
        self.y = self.x = 0
        self.max_y = 0
        self.fg = self.bg = None
        self.bold = self.reverse = False
        self.alt = False

    def put(self, ch):
        if self.x < self.cols:
            self.grid[(self.x, self.y)] = (ch, self.fg, self.bg, self.bold, self.reverse)
        self.max_y = max(self.max_y, self.y)
        self.x += 1
        if self.x >= self.cols:
            # autowrap
            self.x = 0
            self.y += 1

    def feed(self, data):
        i = 0
        n = len(data)
        while i < n:
            c = data[i]
            if c == "\x1b":
                if i + 1 < n and data[i + 1] == "[":
                    j = i + 2
                    while j < n and not data[j].isalpha():
                        j += 1
                    if j >= n:
                        return
                    seq = data[i + 2:j].replace("?", "").replace("$", "").replace("=", "")
                    f = data[j]
                    try:
                        pv = [int(v) if v else 1 for v in seq.split(";")]
                    except ValueError:
                        i = j + 1
                        continue
                    if f == "m":
                        s = strip_sgr(seq)
                        self.fg, self.bg, self.bold, self.reverse = s
                    elif f in ("H", "f"):
                        self.y = max(0, (pv[0] if pv else 1) - 1)
                        self.x = max(0, (pv[1] if len(pv) > 1 else 1) - 1)
                    elif f == "A":
                        step = pv[0] if pv else 1
                        if self.y - step < 0:
                            # The renderer positions relative to the end of the
                            # view; when a large relative up-move would leave
                            # the tracked area, re-anchor to the bottom of the
                            # written content (mirrors the renderer's cursor).
                            if step > 1:
                                self.y = max(0, self.max_y - step)
                            else:
                                self.y = 0
                        else:
                            self.y = max(0, self.y - step)
                    elif f == "B":
                        self.y = min(self.rows - 1, self.y + (pv[0] if pv else 1))
                    elif f == "C":
                        self.x = min(self.cols - 1, self.x + (pv[0] if pv else 1))
                    elif f == "D":
                        self.x = max(0, self.x - (pv[0] if pv else 1))
                    elif f == "d":
                        self.y = max(0, (pv[0] if pv else 1) - 1)
                    elif f == "G":
                        self.x = max(0, (pv[0] if pv else 1) - 1)
                    elif f == "J":
                        if pv[0] == 2:
                            self.grid.clear()
                        elif pv[0] == 1:
                            for k in list(self.grid):
                                if k[1] <= self.y:
                                    self.grid.pop(k, None)
                        else:
                            for k in list(self.grid):
                                if k[1] >= self.y:
                                    self.grid.pop(k, None)
                    elif f == "K":
                        # EL: the omitted parameter defaults to 0 (erase right).
                        kparam = 0 if seq == "" else pv[0]
                        if kparam == 0:
                            for xx in range(self.x, self.cols):
                                self.grid.pop((xx, self.y), None)
                        elif kparam == 1:
                            for xx in range(0, self.x + 1):
                                self.grid.pop((xx, self.y), None)
                        else:
                            for xx in range(self.cols):
                                self.grid.pop((xx, self.y), None)
                    elif f == "h" or f == "l":
                        pass  # mode sets — ignored for screen purposes
                    elif f == "r":
                        pass  # DECSTBM — ignored
                    elif f == "L":
                        # Insert line: shift rows down from y.
                        for yy in range(self.rows - 1, self.y, -1):
                            for xx in range(self.cols):
                                v = self.grid.pop((xx, yy - 1), None)
                                if v is not None:
                                    self.grid[(xx, yy)] = v
                        for xx in range(self.cols):
                            self.grid.pop((xx, self.y), None)
                    elif f == "M":
                        # Delete line: shift rows up from y.
                        for yy in range(self.y, self.rows - 1):
                            for xx in range(self.cols):
                                v = self.grid.pop((xx, yy + 1), None)
                                if v is not None:
                                    self.grid[(xx, yy)] = v
                        for xx in range(self.cols):
                            self.grid.pop((xx, self.rows - 1), None)
                    elif f == "X":
                        # ECH: erase N characters at the cursor (blank them).
                        for xx in range(self.x, min(self.cols, self.x + (pv[0] if pv else 1))):
                            self.grid.pop((xx, self.y), None)
                    i = j + 1
                    continue
                elif i + 1 < n and data[i + 1] == "]":
                    j = i + 2
                    while j < n and data[j] != "\x07":
                        j += 1
                    i = j + 1 if j < n else n
                    continue
                elif i + 1 < n and data[i + 1] == "P":
                    j = i + 2
                    while j < n and not (data[j] == "\x1b" and j + 1 < n and data[j + 1] == "\\"):
                        j += 1
                    i = j + 2 if j < n else n
                    continue
                else:
                    # Non-CSI escapes: ESC-M is the Reverse Index (cursor up
                    # one row, same column) and ESC-D is the Index (cursor
                    # down one row). The renderer uses ESC-M for single-row
                    # up-moves.
                    nxt = data[i + 1] if i + 1 < n else ""
                    if nxt == "M":
                        self.y = max(0, self.y - 1)
                        i += 2
                        continue
                    elif nxt == "D":
                        self.y = min(self.rows - 1, self.y + 1)
                        i += 2
                        continue
                    i += 2
                    continue
            elif c == "\r":
                self.x = 0
            elif c == "\n":
                self.y += 1
                self.x = 0
            elif c == "\b":
                self.x = max(0, self.x - 1)
            elif c == "\t":
                self.x = min(self.cols - 1, self.x + 8 - (self.x % 8))
            elif ord(c) < US and c not in ("\n", "\r", "\b", "\t"):
                pass  # control bytes
            else:
                self.put(c)
            i += 1

    def text(self):
        return "\n".join(
            "".join(self.grid.get((xx, yy), (" ", None, None, False, False))[0]
                    for xx in range(self.cols)).rstrip()
            for yy in range(self.rows)
        )

    def dump(self):
        cells = []
        for (xx, yy), v in sorted(self.grid.items()):
            cells.append([xx, yy, list(v)])
        return {"cells": cells, "rows": self.rows, "cols": self.cols}


def run_spec(cmd, args, spec, gap=0.4, timeout=20.0):
    width = spec.get("width", 80)
    height = spec.get("height", 24)
    pid, master = pty.fork()
    if pid == 0:
        set_win_size(0, width, height)
        os.environ["TERM"] = "xterm-256color"
        os.environ["COLORTERM"] = "truecolor"
        os.execvp(cmd, [cmd] + args)

    set_win_size(master, width, height)
    out = bytearray()
    screens = []
    exited = False
    status = None
    start = time.time()

    def drain(t):
        end = time.time() + t
        while time.time() < end:
            r, _, _ = select.select([master], [], [], 0.05)
            if r:
                try:
                    data = os.read(master, 8192)
                except OSError:
                    return
                if not data:
                    return
                out.extend(data)

    # Initial settle: wait for the program's startup output to become quiet
    # instead of a fixed delay, so cold-started binaries (first example after
    # a build) don't race their first render. Animated examples keep emitting
    # frames, so cap the wait.
    def drain_until_quiet(quiet=0.4, cap=2.0):
        end = time.time() + cap
        last_activity = time.time()
        first_bytes = False
        while time.time() < end:
            if first_bytes and time.time() - last_activity >= quiet:
                return
            r, _, _ = select.select([master], [], [], 0.05)
            if r:
                try:
                    data = os.read(master, 8192)
                except OSError:
                    return
                if not data:
                    return
                out.extend(data)
                first_bytes = True
                last_activity = time.time()

    drain_until_quiet()
    for phase in spec.get("phases", []):
        if "resize" in phase:
            w, h = phase["resize"]
            set_win_size(master, w, h)
            os.write(master, "\x1b[8;%d;%dt".encode() % (h, w))
            drain(phase.get("settle", 0.6))
        if "keys" in phase:
            keys = phase["keys"].encode("latin1").decode("unicode_escape").encode("latin1")
            phases = keys.split(b"|")
            for ph in phases:
                for attempt in range(3):
                    try:
                        os.write(master, ph)
                        break
                    except OSError:
                        time.sleep(0.05)
                if ph is not phases[-1]:
                    time.sleep(gap)
            drain(phase.get("settle", 0.6))
        elif "sleep" in phase:
            drain(phase["sleep"])
        scr = Screen(height, width)
        scr.feed(out.decode("utf-8", "replace"))
        # Spec-declared cells (animated frames, live timers, blinking
        # cursors) are zeroed out of the parity comparison: their exact
        # content is not reproducible even between two upstream Go runs.
        # Phase-level ignore_cells apply only to that phase's screen.
        ignore = spec.get("ignore_cells", []) + phase.get("ignore_cells", [])
        for cell in ignore:
            scr.grid.pop((cell[0], cell[1]), None)
        screens.append(scr.dump())

    # Wait for exit if requested.
    if spec.get("expect_exit", True):
        t0 = time.time()
        while time.time() - t0 < 12:
            wpid, st = os.waitpid(pid, os.WNOHANG)
            if wpid == pid:
                exited = True
                status = st
                break
            r, _, _ = select.select([master], [], [], 0.05)
            if r:
                try:
                    os.read(master, 8192)
                except OSError:
                    # EIO: the child exited and closed the pty; reap it now.
                    wpid, st = os.waitpid(pid, os.WNOHANG)
                    if wpid == pid:
                        exited = True
                        status = st
                    break
        if not exited:
            try:
                os.kill(pid, 9)
            except ProcessLookupError:
                pass
            try:
                os.waitpid(pid, 0)
            except ChildProcessError:
                pass
    else:
        try:
            os.kill(pid, 9)
        except ProcessLookupError:
            pass
        try:
            os.waitpid(pid, 0)
        except ChildProcessError:
            pass
    raw = bytes(out)
    open("/tmp/e2e_last_raw.bin", "wb").write(raw)
    cjk = raw.find("你好".encode("utf-8"))
    return {
        "screens": screens,
        "exited": exited,
        "exit_ok": exited and (status == 0 or os.WIFSIGNALED(status) is False),
        "exit_status": status,
        "raw_tail": raw[-600:].decode("utf-8", "replace"),
        "raw_cjk": (raw[max(0, cjk - 200):cjk + 200].decode("utf-8", "replace") if cjk >= 0 else ""),
    }


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--cmd", required=True)
    p.add_argument("--args", nargs="*", action="append", default=[])
    p.add_argument("--spec", required=True)
    p.add_argument("--out", required=True)
    args = p.parse_args()
    spec = json.load(open(args.spec))
    # Repeated `--args` occurrences (e.g. `--args=-c --args=SCRIPT`) are each
    # a separate group; flatten them into the child argv in order.
    child_args = [a for group in args.args for a in group]
    result = run_spec(args.cmd, child_args, spec)
    with open(args.out, "w") as f:
        json.dump(result, f)


if __name__ == "__main__":
    main()

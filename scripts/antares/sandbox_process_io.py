"""Process supervision, teardown verification, and capped incremental read
for the Antares sandbox (T2c-2).

Split out of sandbox_budget.py's run_budgeted orchestration (T2e-pre, pure
refactor, zero intended behavior change).
"""

from __future__ import annotations

import os
import selectors
import signal
import subprocess
import time

# How long teardown verification waits, after sending the kill signal, before
# concluding the process group is actually gone. Bounded so verification
# itself can never hang or block the caller indefinitely.
_TEARDOWN_GRACE_SECONDS = 1.0
_TEARDOWN_POLL_INTERVAL_SECONDS = 0.05

_READ_CHUNK_BYTES = 4096


def _kill_process_group(process: "subprocess.Popen[bytes]") -> None:
    try:
        os.killpg(os.getpgid(process.pid), signal.SIGKILL)
    except (ProcessLookupError, PermissionError, OSError):
        # The group-kill can fail because the process already exited between
        # this call and the caller's last check (TOCTOU) -- process.kill()
        # can then raise the same "already gone" family of errors for the
        # same reason. That is success for this function's purpose (the
        # process is not running), not a caller-visible crash, so it is
        # caught here exactly as _verify_teardown treats the identical race.
        try:
            process.kill()
        except (ProcessLookupError, PermissionError, OSError):
            pass


def _verify_teardown(process: "subprocess.Popen[bytes]") -> bool:
    """Actively confirm the process group is gone after a kill signal.

    Polls with `os.killpg(pgid, 0)` (signal 0: existence check, sends
    nothing) for up to `_TEARDOWN_GRACE_SECONDS` rather than trusting that
    the kill syscall succeeding means the process has actually exited --
    SIGKILL delivery is not synchronous with process reclamation. Bounded by
    a fixed grace period and a cheap existence probe so verification itself
    cannot hang or reintroduce an unbounded read (the same failure class the
    output cap exists to prevent).
    """
    try:
        pgid = os.getpgid(process.pid)
    except ProcessLookupError:
        return True

    # A stale/reused pgid can make the existence probe raise PermissionError
    # instead of ProcessLookupError on macOS (e.g. the pgid was reassigned to
    # a process owned by another user) -- both mean "we can no longer confirm
    # this is our process group", which is the same as gone for this check.
    deadline = time.monotonic() + _TEARDOWN_GRACE_SECONDS
    while time.monotonic() < deadline:
        try:
            os.killpg(pgid, 0)
        except (ProcessLookupError, PermissionError):
            return True
        time.sleep(_TEARDOWN_POLL_INTERVAL_SECONDS)
    # One last probe after the loop exits: the `while` condition is checked
    # before each poll, so a process that dies in the gap between the final
    # loop iteration's probe and the deadline being crossed would otherwise
    # never get a check performed exactly at expiry. Not reachable in the
    # common case (the loop's own probes already cover it), but cheap enough
    # to keep as the deciding check rather than trust the loop's last result.
    try:
        os.killpg(pgid, 0)
    except (ProcessLookupError, PermissionError):
        return True
    return False


def _close_process_pipes(process: "subprocess.Popen[bytes]") -> None:
    """Close the stdout/stderr pipe fds `Popen` opened for `process`.

    `_read_capped` reads via `os.read` on the raw fds directly rather than
    through `Popen.communicate()`, so nothing else closes these handles.
    Safe to call after any exit path (`process.stdout`/`stderr` are `None`
    only if the caller didn't request pipes, which this module always does).
    """
    if process.stdout is not None:
        process.stdout.close()
    if process.stderr is not None:
        process.stderr.close()


def _read_capped(
    process: "subprocess.Popen[bytes]",
    output_cap_bytes: int,
    timeout_seconds: float,
) -> tuple[bytes, bytes, bool, bool]:
    """Read stdout/stderr incrementally, aborting early on cap or timeout.

    Returns `(stdout, stderr, cap_exceeded, timed_out)`. Never calls
    `process.communicate()`: that call buffers the full stream in memory
    before returning, so a process that writes unbounded output would OOM
    the supervisor before any cap check could fire -- this is the exact
    host-level DoS the phase-1 review flagged. Reads are polled with
    `selectors` so a fixed-size chunk is pulled per iteration and the byte
    total is checked after every chunk, not after the process exits.
    """
    assert process.stdout is not None
    assert process.stderr is not None

    stdout_chunks: list[bytes] = []
    stderr_chunks: list[bytes] = []
    total_bytes = 0
    cap_exceeded = False
    timed_out = False

    sel = selectors.DefaultSelector()
    sel.register(process.stdout, selectors.EVENT_READ, "stdout")
    sel.register(process.stderr, selectors.EVENT_READ, "stderr")
    open_streams = 2
    deadline = time.monotonic() + timeout_seconds

    try:
        while open_streams > 0:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                timed_out = True
                break
            for key, _ in sel.select(timeout=min(remaining, 0.1)):
                chunk = os.read(key.fileobj.fileno(), _READ_CHUNK_BYTES)
                if not chunk:
                    sel.unregister(key.fileobj)
                    open_streams -= 1
                    continue
                total_bytes += len(chunk)
                if key.data == "stdout":
                    stdout_chunks.append(chunk)
                else:
                    stderr_chunks.append(chunk)
                if total_bytes > output_cap_bytes:
                    cap_exceeded = True
                    break
            if cap_exceeded:
                break
            if process.poll() is not None and open_streams > 0:
                # The direct child has exited, but a grandchild it spawned
                # (e.g. `yes` in `yes | head`) can still hold a pipe's write
                # end open, so a plain blocking os.read here can hang
                # forever waiting for data that will never come -- exactly
                # the unbounded-read failure mode this whole function exists
                # to prevent. Drain with the same selector-timeout pattern as
                # the main loop instead of assuming EOF is imminent.
                drain_deadline = time.monotonic() + 0.1
                while open_streams > 0 and time.monotonic() < drain_deadline:
                    for key, _ in sel.select(timeout=0.02):
                        chunk = os.read(key.fileobj.fileno(), _READ_CHUNK_BYTES)
                        if not chunk:
                            sel.unregister(key.fileobj)
                            open_streams -= 1
                            continue
                        total_bytes += len(chunk)
                        if key.data == "stdout":
                            stdout_chunks.append(chunk)
                        else:
                            stderr_chunks.append(chunk)
                        if total_bytes > output_cap_bytes:
                            cap_exceeded = True
                            break
                    if cap_exceeded:
                        break
                break
    finally:
        sel.close()

    return b"".join(stdout_chunks), b"".join(stderr_chunks), cap_exceeded, timed_out

"""RLIMIT/Darwin-detection concern for the Antares sandbox (T2c-2).

Split out of sandbox_budget.py's run_budgeted orchestration (T2e-pre, pure
refactor, zero intended behavior change): owns whether this platform can
enforce every resource cap T2c-2 promises, and composes the single
preexec_fn subprocess.Popen accepts from that decision plus T2c-1's
privilege drop.
"""

from __future__ import annotations

import importlib.util
import os
import platform
import resource
import sys
from pathlib import Path
from typing import Callable


def _load_sibling_module(module_name: str, filename: str):
    """Load `filename` as `module_name`, reusing an already-loaded copy from
    `sys.modules` if one exists.

    Required once a single concern is split across sibling files that must
    share one class identity for Enum/dataclass types defined elsewhere
    (e.g. `TerminalStateKind`): `importlib.util.module_from_spec` +
    `exec_module` always re-executes a file from scratch and does not
    consult `sys.modules` on its own, so two independent loads of the same
    file produce two distinct, non-`==`-comparable class objects. See
    docs/tasks/antares-security-specialist-advisor.md § T2e-pre EC-4.
    """
    if module_name in sys.modules:
        return sys.modules[module_name]
    script = Path(__file__).with_name(filename)
    spec = importlib.util.spec_from_file_location(module_name, script)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"Unable to load script spec for {script}")
    mod = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = mod
    spec.loader.exec_module(mod)
    return mod


_SANDBOX_RUNNER_MOD = _load_sibling_module("antares_sandbox_runner", "sandbox_runner.py")
_drop_privileges = _SANDBOX_RUNNER_MOD._drop_privileges


def _resource_limits_available() -> bool:
    """Whether the POSIX RLIMIT primitives this module needs actually work.

    Mirrors T2c-1's network-isolation fail-closed precedent: if the platform
    cannot enforce every resource cap this module promises, refuse to run
    rather than execute with a silently unenforced cap.

    Darwin is unconditionally excluded. Two independent RLIMITs are
    unenforceable there for the sandbox's purposes, confirmed empirically
    during T2c-2 implementation:

    - `RLIMIT_AS` (address-space / RAM ceiling): not reliably settable at
      all -- `setrlimit(RLIMIT_AS, ...)` fails even in the parent process on
      current macOS, not just inside a `preexec_fn`.
    - `RLIMIT_NPROC` (process-count ceiling): settable, but scoped to the
      *entire UID* system-wide on BSD-derived kernels, not to the sandboxed
      command's own process tree. A cap tight enough to matter (e.g. 16)
      breaks trivial multi-process pipelines outright, because the real
      user account already runs far more than that system-wide; a cap loose
      enough not to break normal usage (e.g. 2048) does not bound anything
      the sandboxed command could actually do, since the account's ambient
      process count already approaches that range on a live desktop. There
      is no value that is both a real per-command bound and compatible with
      an ordinary shell pipeline on this platform.

    Claiming "resource limits enforced" while a promised cap is either
    unsettable or accidental is exactly the silent-degradation failure mode
    T2c-1 already rejected for network isolation, so the whole session fails
    closed on Darwin instead of enforcing a partial or fake set of caps.
    """
    if platform.system() == "Darwin":
        return False
    return (
        os.name == "posix"
        and hasattr(resource, "RLIMIT_CPU")
        and hasattr(resource, "RLIMIT_AS")
        and hasattr(resource, "RLIMIT_NPROC")
    )


def _compose_preexec(
    cpu_seconds: int,
    address_space_bytes: int,
    max_processes: int,
) -> Callable[[], None]:
    """Build the single `preexec_fn` passed to `Popen`.

    `subprocess.Popen` accepts exactly one `preexec_fn`; T2c-1's
    `_drop_privileges` and T2c-2's RLIMIT calls must therefore be composed
    into one function rather than passed separately. Ordering matters:
    `setrlimit` runs first, while the child may still hold whatever
    privileges the host process started with, so a limit that requires
    elevated privilege to raise (it never needs privilege to *lower*) is not
    silently rejected by having already dropped to an unprivileged uid.
    Privilege drop runs second and is irreversible for the remaining process
    lifetime, which is the correct order for a security boundary: tighten
    resource limits, then relinquish the privilege that could otherwise
    loosen them again.
    """
    drop_privileges = _drop_privileges()

    def _preexec() -> None:  # pragma: no cover - runs only inside the child
        resource.setrlimit(resource.RLIMIT_CPU, (cpu_seconds, cpu_seconds))
        resource.setrlimit(
            resource.RLIMIT_AS, (address_space_bytes, address_space_bytes)
        )
        resource.setrlimit(resource.RLIMIT_NPROC, (max_processes, max_processes))
        drop_privileges()

    return _preexec

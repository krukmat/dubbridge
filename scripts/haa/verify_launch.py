#!/Library/DubBridgeHAA/runtime/bin/python3 -I
"""Install as root-owned bin/verify-launch only after owner review.

Runs as the dedicated non-root verifier, never root. No command-line
arguments, no shell, no credential or user-owned Python dependencies.
The stdin request is admitted by the trusted executor's narrow sudo rule.
"""
import json
import os
from pathlib import Path
import shutil
import stat
import sys
import tempfile

BASE=Path("/Library/DubBridgeHAA")
EXPORTS=BASE/"exports"
WORK=BASE/"verifier-work"
PATH="/Library/DubBridgeHAA/runtime/bin:/usr/bin:/bin"


def main():
    if len(sys.argv)!=1 or os.getuid()==0 or os.geteuid()==0:
        return 125
    raw=sys.stdin.buffer.read(65537)
    if not 0<len(raw)<=65536:
        return 125
    request=json.loads(raw)
    if (not isinstance(request,dict) or set(request)!={"schema","source","argv",
            "executor_uid","verifier_uid","verifier_gid"}
            or request["schema"]!="dubbridge.verifier-launch.v1"):
        return 125
    executor,uid,gid=(request[k] for k in ("executor_uid","verifier_uid","verifier_gid"))
    if (any(type(v) is not int or v<=0 for v in (executor,uid,gid))
            or executor==uid or os.getuid()!=uid or os.geteuid()!=uid
            or os.getgid()!=gid or os.getegid()!=gid):
        return 125
    source=Path(request["source"])
    if source!=source.resolve() or source.parent!=EXPORTS or not source.name.startswith("verify-"):
        return 125
    s=source.lstat()
    if (not stat.S_ISDIR(s.st_mode) or s.st_uid!=executor or s.st_gid!=gid
            or stat.S_IMODE(s.st_mode)!=0o750 or os.access(source,os.W_OK)):
        return 125
    w=WORK.lstat()
    if (WORK!=WORK.resolve() or not stat.S_ISDIR(w.st_mode) or w.st_uid!=uid
            or stat.S_IMODE(w.st_mode)!=0o700):
        return 125
    argv=request["argv"]
    if (not isinstance(argv,list) or not argv or len(argv)>128
            or any(not isinstance(v,str) or not v or "\x00" in v for v in argv)):
        return 125
    # Resolve executables using the trusted runtime, system tools or the
    # exact exported tree. No model-controlled PATH or shell evaluation.
    command=argv[0]
    if "/" in command:
        executable=Path(command) if command.startswith("/") else source/command
        executable=executable.resolve()
        if not (executable.is_relative_to(source)
                or any(executable.is_relative_to(Path(p)) for p in PATH.split(":"))):
            return 125
        executable=str(executable)
    else:
        executable=shutil.which(command,path=PATH)
        if executable is None:
            return 125
    os.umask(0o077)
    scratch=tempfile.mkdtemp(prefix="run-",dir=WORK)
    env={"PATH":PATH,"HOME":scratch,"TMPDIR":scratch,"TMP":scratch,"TEMP":scratch,
         "LANG":"C","LC_ALL":"C","PYTHONDONTWRITEBYTECODE":"1",
         "XDG_CACHE_HOME":scratch}
    os.chdir(source)
    # No wrapper-generated PASS receipt: sudo observes the actual exit
    # status of the process replacing this helper. Scratch is retained for
    # explicit controlled cleanup; no reuse across commands.
    os.execve(executable,[executable,*argv[1:]],env)


if __name__=="__main__":
    try:
        raise SystemExit(main())
    except Exception:
        raise SystemExit(125)

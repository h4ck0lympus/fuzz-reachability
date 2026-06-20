"""C / C++ bitcode acquisition via gllvm.

Build the project with the gclang/gclang++ wrappers (which embed bitcode-path
metadata into each object), then run get-bc on the built artifact to extract a
whole-program .bc. The artifact is auto-detected from the build output when not
given explicitly. Independent of the project's own LTO setup.
"""

import mmap
import os
import shutil
import struct
import subprocess
import time


class AcquireError(RuntimeError):
    pass


_SKIP_DIRS = {".git", ".hg", ".svn", "node_modules", "__pycache__", ".venv",
              ".cache"}
_BC_MARKERS = (b".llvm_bc", b"__llvm_bc")
_KIND_RANK = {"exec": 3, "shared": 2, "archive": 1, "object": 0}
_MACHO_MAGIC = (0xFEEDFACE, 0xFEEDFACF, 0xCEFAEDFE, 0xCFFAEDFE, 0xCAFEBABE)


def _build_env(clang_bindir: str) -> dict:
    env = dict(os.environ)
    env["CC"] = "gclang"
    env["CXX"] = "gclang++"
    env["LLVM_COMPILER_PATH"] = clang_bindir
    return env


def detect_build_cmd(project_dir):
    """Pick a build command for a C/C++ project by probing for the well-known
    build files, in the order configure -> make -> cmake -> ninja -> meson, with
    an autotools-bootstrap fallback. Returns a shell command string, or None if
    nothing is recognized (the caller then falls back to plain `make`). The
    gllvm wrappers are injected via CC/CXX, which every build system below
    honours at configure time, so the chosen command embeds bitcode regardless.
    """
    def has(*names):
        return any(os.path.exists(os.path.join(project_dir, n)) for n in names)

    if has("configure"):
        return "./configure && make"
    if has("Makefile", "makefile", "GNUmakefile"):
        return "make"
    if has("CMakeLists.txt"):
        return "cmake -S . -B build && cmake --build build"
    if has("build.ninja"):
        return "ninja"
    if has("meson.build"):
        return "meson setup build && ninja -C build"
    if has("autogen.sh"):
        return "./autogen.sh && ./configure && make"
    if has("configure.ac", "configure.in"):
        return "autoreconf -i && ./configure && make"
    return None


def _has_bitcode_marker(path):
    """True if `path` embeds gllvm's bitcode-section name (so get-bc can read it)."""
    try:
        with open(path, "rb") as fh:
            mm = mmap.mmap(fh.fileno(), 0, access=mmap.ACCESS_READ)
            try:
                return any(mm.find(m) != -1 for m in _BC_MARKERS)
            finally:
                mm.close()
    except (OSError, ValueError):
        return False


def _kind_by_ext(path):
    n = os.path.basename(path)
    if n.endswith(".dylib"):
        return "shared"
    if n.endswith(".o"):
        return "object"
    return "exec"


def _classify(path):
    """Return the artifact kind (exec/shared/archive/object) for a built file, or
    None if it is not something get-bc could read."""
    try:
        with open(path, "rb") as fh:
            hdr = fh.read(20)
    except OSError:
        return None
    if len(hdr) < 8:
        return None
    if hdr[:8] == b"!<arch>\n":
        return "archive"
    if hdr[:4] == b"\x7fELF":
        endian = "<H" if hdr[5] != 2 else ">H"
        etype = struct.unpack_from(endian, hdr, 16)[0]
        if etype == 1:
            return "object"
        if etype == 2:
            return "exec"
        if etype == 3:
            name = os.path.basename(path)
            if name.endswith(".so") or ".so." in name:
                return "shared"
            return "exec" if os.access(path, os.X_OK) else "shared"
        return None
    if struct.unpack_from("<I", hdr, 0)[0] in _MACHO_MAGIC:
        return _kind_by_ext(path)
    return None


def find_artifacts(project_dir, newer_than=None):
    """Walk `project_dir` for built files get-bc can read, ranked best-first:
    files carrying gllvm's bitcode section come first, then ones built by this
    run, then by kind (executable > shared lib > archive > object), then newest.
    """
    cands = []
    for root, dirs, files in os.walk(project_dir):
        dirs[:] = [d for d in dirs if d not in _SKIP_DIRS]
        for f in files:
            p = os.path.join(root, f)
            if os.path.islink(p) or not os.path.isfile(p):
                continue
            kind = _classify(p)
            if kind is None:
                continue
            try:
                mtime = os.path.getmtime(p)
            except OSError:
                continue
            fresh = newer_than is not None and mtime >= newer_than - 2
            cands.append((_has_bitcode_marker(p), fresh, _KIND_RANK[kind], mtime, p))
    cands.sort(reverse=True)
    return [c[-1] for c in cands]


def acquire_c_bitcode(project_dir, tc, artifact=None, build_cmd=None):
    """Build `project_dir` with gllvm wrappers and extract its whole-program .bc.

    artifact: path (relative to project_dir) of the built binary/object/archive.
    When None (or it does not exist after the build), the build product is
    auto-detected. Returns the absolute path to the extracted .bc.
    """
    if not shutil.which("gclang"):
        raise AcquireError("gclang not found on PATH; run scripts/setup.sh")
    clang_bindir = os.path.dirname(os.path.abspath(tc.clang))
    env = _build_env(clang_bindir)
    cmd = build_cmd or ["make"]
    before = time.time()
    r = subprocess.run(cmd, cwd=project_dir, env=env, capture_output=True, text=True)
    if r.returncode != 0:
        raise AcquireError(f"build failed:\n{r.stdout}\n{r.stderr}")

    explicit = os.path.join(project_dir, artifact) if artifact else None
    if explicit and os.path.exists(explicit):
        candidates = [explicit]
    else:
        if explicit:
            print(f"warning: --artifact {artifact!r} not found after build; "
                  "auto-detecting the build product")
        candidates = find_artifacts(project_dir, newer_than=before)
    if not candidates:
        raise AcquireError(
            f"no build artifact with embedded bitcode found under {project_dir}; "
            "pass --artifact PATH to the built binary/object/archive")

    errors = []
    for art in candidates[:8]:
        out = art + ".bc"
        r = subprocess.run(
            ["get-bc", "-o", out, art], cwd=project_dir, capture_output=True, text=True
        )
        if r.returncode == 0 and os.path.exists(out) and os.path.getsize(out) > 0:
            if not (explicit and art == explicit):
                print(f"artifact: {os.path.relpath(art, project_dir)}")
            return out
        errors.append(f"{os.path.relpath(art, project_dir)}: {r.stderr.strip()}")
    raise AcquireError(
        "get-bc could not extract bitcode from any detected artifact:\n  "
        + "\n  ".join(errors))

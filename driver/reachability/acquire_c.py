"""C / C++ bitcode acquisition via gllvm.

Build the project with the gclang/gclang++ wrappers (which embed bitcode-path
metadata into each object), then run get-bc on the final artifact to extract a
whole-program .bc. Independent of the project's own LTO setup.
"""

import os
import shutil
import subprocess


class AcquireError(RuntimeError):
    pass


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


def acquire_c_bitcode(project_dir, tc, artifact, build_cmd=None):
    """Build `project_dir` with gllvm wrappers and extract `<artifact>.bc`.

    artifact: path (relative to project_dir) of the built binary/archive.
    Returns the absolute path to the extracted .bc.
    """
    if not shutil.which("gclang"):
        raise AcquireError("gclang not found on PATH; run scripts/setup.sh")
    clang_bindir = os.path.dirname(os.path.abspath(tc.clang))
    env = _build_env(clang_bindir)
    cmd = build_cmd or ["make"]
    r = subprocess.run(cmd, cwd=project_dir, env=env, capture_output=True, text=True)
    if r.returncode != 0:
        raise AcquireError(f"build failed:\n{r.stdout}\n{r.stderr}")
    art = os.path.join(project_dir, artifact)
    out = art + ".bc"
    r = subprocess.run(
        ["get-bc", "-o", out, art], cwd=project_dir, capture_output=True, text=True
    )
    if r.returncode != 0:
        raise AcquireError(f"get-bc failed:\n{r.stdout}\n{r.stderr}")
    return out

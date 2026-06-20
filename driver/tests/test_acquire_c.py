from reachability import acquire_c


def test_build_env_sets_wrappers(monkeypatch):
    monkeypatch.setenv("PATH", "/usr/bin")
    env = acquire_c._build_env(clang_bindir="/usr/lib/llvm-21/bin")
    assert env["CC"].endswith("gclang")
    assert env["CXX"].endswith("gclang++")
    assert env["LLVM_COMPILER_PATH"] == "/usr/lib/llvm-21/bin"


def test_detect_build_cmd_none(tmp_path):
    assert acquire_c.detect_build_cmd(str(tmp_path)) is None


def test_detect_build_cmd_make(tmp_path):
    (tmp_path / "Makefile").write_text("all:\n\ttrue\n")
    assert acquire_c.detect_build_cmd(str(tmp_path)) == "make"


def test_detect_build_cmd_cmake(tmp_path):
    (tmp_path / "CMakeLists.txt").write_text("project(x)\n")
    assert acquire_c.detect_build_cmd(str(tmp_path)) == (
        "cmake -S . -B build && cmake --build build"
    )


def test_detect_build_cmd_meson(tmp_path):
    (tmp_path / "meson.build").write_text("project('x', 'c')\n")
    assert acquire_c.detect_build_cmd(str(tmp_path)) == (
        "meson setup build && ninja -C build"
    )


def test_detect_build_cmd_ninja(tmp_path):
    (tmp_path / "build.ninja").write_text("rule cc\n")
    assert acquire_c.detect_build_cmd(str(tmp_path)) == "ninja"


def test_detect_build_cmd_configure_precedes_make(tmp_path):
    (tmp_path / "configure").write_text("#!/bin/sh\n")
    (tmp_path / "Makefile").write_text("all:\n\ttrue\n")
    assert acquire_c.detect_build_cmd(str(tmp_path)) == "./configure && make"

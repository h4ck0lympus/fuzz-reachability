"""Stage 2: merge collected .bc into one module via llvm-link."""

import subprocess


class LinkError(RuntimeError):
    pass


def link_bitcode(bc_paths, out_path, tc):
    """llvm-link all `bc_paths` into `out_path`. Returns out_path.

    The first module is linked positionally and the rest via llvm-link
    --override, so a symbol defined in more than one module (e.g. an object
    archived into several static libraries, like libport inside both libtiff.a
    and libtiffxx.a) overrides the earlier definition instead of aborting the
    link with a duplicate-symbol error.
    """
    if not bc_paths:
        raise LinkError("no bitcode files to link")
    first, *rest = bc_paths
    cmd = [tc.llvm_link, first, *(f"--override={p}" for p in rest), "-o", out_path]
    r = subprocess.run(cmd, capture_output=True, text=True)
    if r.returncode != 0:
        raise LinkError(f"llvm-link failed:\n{r.stderr}")
    return out_path

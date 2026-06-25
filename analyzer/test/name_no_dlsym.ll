; Gate: a function name appearing as a string constant adds nothing when the
; module performs no dynamic symbol lookup (no dlsym/dlopen-family call). This
; keeps the heuristic inert for programs that never resolve symbols by name.

@.dynname = private unnamed_addr constant [11 x i8] c"dyn_target\00"

define void @dyn_target() {
  ret void
}

define i32 @entry() {
  ret i32 0
}

; dlsym-by-name reachability.
; `entry` resolves a symbol by NAME via dlsym and calls the result. `dyn_target`
; is externally visible and never has its address taken, so the only thing
; linking it to the program is the string constant holding its name -- exactly
; the pattern the direct/indirect/escape edge builders cannot see.
;
; Negative controls:
;   exported_unused  - externally visible but its name is not a string constant.
;   internal_named   - its name IS a string, but it has internal linkage, so it
;                      is not in the dynamic symbol table dlsym can resolve.

@.dynname = private unnamed_addr constant [11 x i8] c"dyn_target\00"
@.intname = private unnamed_addr constant [15 x i8] c"internal_named\00"

declare ptr @dlopen(ptr, i32)
declare ptr @dlsym(ptr, ptr)

define void @dyn_target() {
  ret void
}

define void @exported_unused() {
  ret void
}

define internal void @internal_named() {
  ret void
}

define i32 @entry() {
  %h = call ptr @dlopen(ptr null, i32 0)
  %p = call ptr @dlsym(ptr %h, ptr @.dynname)
  call void %p()
  ret i32 0
}

; Reachability confidence tiers in the JSON report.
;   direct_leaf : reached by a direct call                         -> high
;   cb_target   : address escapes to an external function          -> medium
;   real_target : address flows through a global to an indirect callee -> medium
;   decoy       : address-taken and type-matches the live indirect call, but its
;                 address only sinks into inline asm (black_box-style)   -> low
; decoy is still reachable (the type-based resolver keeps it); confidence only
; annotates that there is no value-flow evidence it is actually callable.

@table = global ptr @real_target

declare void @register(ptr)

define void @direct_leaf() {
  ret void
}

define void @cb_target() {
  ret void
}

define void @real_target() {
  ret void
}

define void @decoy() {
  ret void
}

define void @entry() {
  call void @direct_leaf()
  call void @register(ptr @cb_target)
  %f = load ptr, ptr @table
  call void %f()
  %i = ptrtoint ptr @decoy to i64
  call void asm sideeffect "", "r"(i64 %i)
  ret void
}

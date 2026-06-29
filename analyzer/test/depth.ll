define void @target() {
  ret void
}

define void @mid() {
  call void @target()
  ret void
}

define void @entry() {
  call void @mid()
  call void @target()
  ret void
}

declare void @memcpy(ptr, ptr, i64)

define void @harness(ptr %p, i64 %n) {
entry:
  call void @memcpy(ptr %p, ptr %p, i64 %n)
  %cmp = icmp eq i64 %n, 0
  br i1 %cmp, label %t, label %f
t:
  call void @a(ptr %p)
  br label %f
f:
  call void @b(i64 %n)
  ret void
}

define void @a(ptr %p) {
entry:
  br label %loop
loop:
  %i = phi i64 [ 0, %entry ], [ %i2, %loop ]
  call void @c(ptr %p)
  %i2 = add i64 %i, 1
  %cc = icmp slt i64 %i2, 10
  br i1 %cc, label %loop, label %done
done:
  ret void
}

define void @b(i64 %x) {
  call void @d(ptr null)
  ret void
}

define void @c(ptr %p) {
  %buf = alloca [16 x i8]
  %v = alloca i32
  ret void
}

define void @d(ptr %p) {
  ret void
}

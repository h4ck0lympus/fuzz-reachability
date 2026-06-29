; ModuleID = '/tmp/claude-1000/-prg-fuzz-reachability/705378ba-e53f-4393-ac37-f795cc5096b1/scratchpad/proc.c'
source_filename = "/tmp/claude-1000/-prg-fuzz-reachability/705378ba-e53f-4393-ac37-f795cc5096b1/scratchpad/proc.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

; Function Attrs: noinline nounwind optnone uwtable
define dso_local void @process(ptr noundef %0, i32 noundef %1) #0 !dbg !9 {
  %3 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca [64 x i8], align 16
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  store ptr %0, ptr %3, align 8
    #dbg_declare(ptr %3, !17, !DIExpression(), !18)
  store i32 %1, ptr %4, align 4
    #dbg_declare(ptr %4, !19, !DIExpression(), !20)
    #dbg_declare(ptr %5, !21, !DIExpression(), !25)
    #dbg_declare(ptr %6, !26, !DIExpression(), !27)
    #dbg_declare(ptr %7, !28, !DIExpression(), !30)
  store i32 0, ptr %7, align 4, !dbg !30
  store i32 0, ptr %6, align 4, !dbg !31
  br label %8, !dbg !33

8:                                                ; preds = %21, %2
  %9 = load i32, ptr %6, align 4, !dbg !34
  %10 = load i32, ptr %4, align 4, !dbg !36
  %11 = icmp ult i32 %9, %10, !dbg !37
  br i1 %11, label %12, label %24, !dbg !38

12:                                               ; preds = %8
  %13 = load ptr, ptr %3, align 8, !dbg !39
  %14 = load i32, ptr %6, align 4, !dbg !40
  %15 = zext i32 %14 to i64, !dbg !39
  %16 = getelementptr inbounds nuw i8, ptr %13, i64 %15, !dbg !39
  %17 = load i8, ptr %16, align 1, !dbg !39
  %18 = sext i8 %17 to i32, !dbg !39
  %19 = load i32, ptr %7, align 4, !dbg !41
  %20 = add nsw i32 %19, %18, !dbg !41
  store i32 %20, ptr %7, align 4, !dbg !41
  br label %21, !dbg !42

21:                                               ; preds = %12
  %22 = load i32, ptr %6, align 4, !dbg !43
  %23 = add i32 %22, 1, !dbg !43
  store i32 %23, ptr %6, align 4, !dbg !43
  br label %8, !dbg !44, !llvm.loop !45

24:                                               ; preds = %8
  %25 = getelementptr inbounds [64 x i8], ptr %5, i64 0, i64 0, !dbg !48
  %26 = load ptr, ptr %3, align 8, !dbg !49
  %27 = load i32, ptr %4, align 4, !dbg !50
  %28 = zext i32 %27 to i64, !dbg !50
  call void @llvm.memcpy.p0.p0.i64(ptr align 16 %25, ptr align 1 %26, i64 %28, i1 false), !dbg !48
  %29 = getelementptr inbounds [64 x i8], ptr %5, i64 0, i64 0, !dbg !51
  call void @sink(ptr noundef %29), !dbg !52
  ret void, !dbg !53
}

; Function Attrs: nocallback nofree nosync nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i64(ptr noalias writeonly captures(none), ptr noalias readonly captures(none), i64, i1 immarg) #1

declare void @sink(ptr noundef) #2

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { nocallback nofree nosync nounwind willreturn memory(argmem: readwrite) }
attributes #2 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!2, !3, !4, !5, !6, !7}
!llvm.ident = !{!8}

!0 = distinct !DICompileUnit(language: DW_LANG_C11, file: !1, producer: "Ubuntu clang version 23.0.0 (++20260621082240+441725611d0e-1~exp1~20260621082250.1677)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, nameTableKind: None)
!1 = !DIFile(filename: "/tmp/claude-1000/-prg-fuzz-reachability/705378ba-e53f-4393-ac37-f795cc5096b1/scratchpad/proc.c", directory: "/prg/fuzz-reachability", checksumkind: CSK_MD5, checksum: "06170c08cc75bf95694dc2d2e294afec")
!2 = !{i32 7, !"Dwarf Version", i32 5}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !{i32 8, !"PIC Level", i32 2}
!5 = !{i32 7, !"PIE Level", i32 2}
!6 = !{i32 7, !"uwtable", i32 2}
!7 = !{i32 7, !"frame-pointer", i32 2}
!8 = !{!"Ubuntu clang version 23.0.0 (++20260621082240+441725611d0e-1~exp1~20260621082250.1677)"}
!9 = distinct !DISubprogram(name: "process", scope: !10, file: !10, line: 3, type: !11, scopeLine: 3, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0, retainedNodes: !16)
!10 = !DIFile(filename: "/tmp/claude-1000/-prg-fuzz-reachability/705378ba-e53f-4393-ac37-f795cc5096b1/scratchpad/proc.c", directory: "", checksumkind: CSK_MD5, checksum: "06170c08cc75bf95694dc2d2e294afec")
!11 = !DISubroutineType(types: !12)
!12 = !{null, !13, !15}
!13 = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: !14, size: 64)
!14 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_signed_char)
!15 = !DIBasicType(name: "unsigned int", size: 32, encoding: DW_ATE_unsigned)
!16 = !{}
!17 = !DILocalVariable(name: "p", arg: 1, scope: !9, file: !10, line: 3, type: !13)
!18 = !DILocation(line: 3, column: 20, scope: !9)
!19 = !DILocalVariable(name: "n", arg: 2, scope: !9, file: !10, line: 3, type: !15)
!20 = !DILocation(line: 3, column: 32, scope: !9)
!21 = !DILocalVariable(name: "buf", scope: !9, file: !10, line: 4, type: !22)
!22 = !DICompositeType(tag: DW_TAG_array_type, baseType: !14, size: 512, elements: !23)
!23 = !{!24}
!24 = !DISubrange(count: 64)
!25 = !DILocation(line: 4, column: 8, scope: !9)
!26 = !DILocalVariable(name: "i", scope: !9, file: !10, line: 5, type: !15)
!27 = !DILocation(line: 5, column: 12, scope: !9)
!28 = !DILocalVariable(name: "total", scope: !9, file: !10, line: 6, type: !29)
!29 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!30 = !DILocation(line: 6, column: 7, scope: !9)
!31 = !DILocation(line: 7, column: 10, scope: !32)
!32 = distinct !DILexicalBlock(scope: !9, file: !10, line: 7, column: 3)
!33 = !DILocation(line: 7, column: 8, scope: !32)
!34 = !DILocation(line: 7, column: 15, scope: !35)
!35 = distinct !DILexicalBlock(scope: !32, file: !10, line: 7, column: 3)
!36 = !DILocation(line: 7, column: 19, scope: !35)
!37 = !DILocation(line: 7, column: 17, scope: !35)
!38 = !DILocation(line: 7, column: 3, scope: !32)
!39 = !DILocation(line: 7, column: 36, scope: !35)
!40 = !DILocation(line: 7, column: 38, scope: !35)
!41 = !DILocation(line: 7, column: 33, scope: !35)
!42 = !DILocation(line: 7, column: 27, scope: !35)
!43 = !DILocation(line: 7, column: 23, scope: !35)
!44 = !DILocation(line: 7, column: 3, scope: !35)
!45 = distinct !{!45, !38, !46, !47}
!46 = !DILocation(line: 7, column: 39, scope: !32)
!47 = !{!"llvm.loop.mustprogress"}
!48 = !DILocation(line: 8, column: 3, scope: !9)
!49 = !DILocation(line: 8, column: 15, scope: !9)
!50 = !DILocation(line: 8, column: 18, scope: !9)
!51 = !DILocation(line: 9, column: 8, scope: !9)
!52 = !DILocation(line: 9, column: 3, scope: !9)
!53 = !DILocation(line: 10, column: 1, scope: !9)

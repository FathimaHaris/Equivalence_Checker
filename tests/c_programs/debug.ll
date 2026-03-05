; ModuleID = 'test.c'
source_filename = "test.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

@.str = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1, !dbg !0

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @test(i32 noundef %0) #0 !dbg !17 {
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  store i32 %0, ptr %3, align 4
  call void @llvm.dbg.declare(metadata ptr %3, metadata !22, metadata !DIExpression()), !dbg !23
  store i32 5, ptr %3, align 4, !dbg !24
  call void @llvm.dbg.declare(metadata ptr %4, metadata !25, metadata !DIExpression()), !dbg !26
  %5 = load i32, ptr %3, align 4, !dbg !27
  %6 = add nsw i32 %5, 3, !dbg !28
  store i32 %6, ptr %4, align 4, !dbg !26
  %7 = load i32, ptr %4, align 4, !dbg !29
  %8 = icmp sgt i32 %7, 10, !dbg !31
  br i1 %8, label %9, label %10, !dbg !32

9:                                                ; preds = %1
  store i32 1, ptr %2, align 4, !dbg !33
  br label %11, !dbg !33

10:                                               ; preds = %1
  store i32 2, ptr %2, align 4, !dbg !35
  br label %11, !dbg !35

11:                                               ; preds = %10, %9
  %12 = load i32, ptr %2, align 4, !dbg !37
  ret i32 %12, !dbg !37
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare void @llvm.dbg.declare(metadata, metadata, metadata) #1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local i32 @main() #0 !dbg !38 {
  %1 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  %2 = call i32 @test(i32 noundef 10), !dbg !41
  %3 = call i32 (ptr, ...) @printf(ptr noundef @.str, i32 noundef %2), !dbg !42
  ret i32 0, !dbg !43
}

declare i32 @printf(ptr noundef, ...) #2

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }

!llvm.dbg.cu = !{!7}
!llvm.module.flags = !{!9, !10, !11, !12, !13, !14, !15}
!llvm.ident = !{!16}

!0 = !DIGlobalVariableExpression(var: !1, expr: !DIExpression())
!1 = distinct !DIGlobalVariable(scope: null, file: !2, line: 29, type: !3, isLocal: true, isDefinition: true)
!2 = !DIFile(filename: "test.c", directory: "/home/fathima/equivalence_checker/tests/c_programs", checksumkind: CSK_MD5, checksum: "8bc161f5fc26b6ad58282c3cfa4e347e")
!3 = !DICompositeType(tag: DW_TAG_array_type, baseType: !4, size: 32, elements: !5)
!4 = !DIBasicType(name: "char", size: 8, encoding: DW_ATE_signed_char)
!5 = !{!6}
!6 = !DISubrange(count: 4)
!7 = distinct !DICompileUnit(language: DW_LANG_C11, file: !2, producer: "Ubuntu clang version 18.1.3 (1ubuntu1)", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, globals: !8, splitDebugInlining: false, nameTableKind: None)
!8 = !{!0}
!9 = !{i32 7, !"Dwarf Version", i32 5}
!10 = !{i32 2, !"Debug Info Version", i32 3}
!11 = !{i32 1, !"wchar_size", i32 4}
!12 = !{i32 8, !"PIC Level", i32 2}
!13 = !{i32 7, !"PIE Level", i32 2}
!14 = !{i32 7, !"uwtable", i32 2}
!15 = !{i32 7, !"frame-pointer", i32 2}
!16 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
!17 = distinct !DISubprogram(name: "test", scope: !2, file: !2, line: 14, type: !18, scopeLine: 15, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !7, retainedNodes: !21)
!18 = !DISubroutineType(types: !19)
!19 = !{!20, !20}
!20 = !DIBasicType(name: "int", size: 32, encoding: DW_ATE_signed)
!21 = !{}
!22 = !DILocalVariable(name: "x", arg: 1, scope: !17, file: !2, line: 14, type: !20)
!23 = !DILocation(line: 14, column: 14, scope: !17)
!24 = !DILocation(line: 16, column: 6, scope: !17)
!25 = !DILocalVariable(name: "y", scope: !17, file: !2, line: 17, type: !20)
!26 = !DILocation(line: 17, column: 9, scope: !17)
!27 = !DILocation(line: 17, column: 11, scope: !17)
!28 = !DILocation(line: 17, column: 12, scope: !17)
!29 = !DILocation(line: 18, column: 9, scope: !30)
!30 = distinct !DILexicalBlock(scope: !17, file: !2, line: 18, column: 9)
!31 = !DILocation(line: 18, column: 10, scope: !30)
!32 = !DILocation(line: 18, column: 9, scope: !17)
!33 = !DILocation(line: 20, column: 9, scope: !34)
!34 = distinct !DILexicalBlock(scope: !30, file: !2, line: 19, column: 5)
!35 = !DILocation(line: 24, column: 9, scope: !36)
!36 = distinct !DILexicalBlock(scope: !30, file: !2, line: 23, column: 5)
!37 = !DILocation(line: 26, column: 1, scope: !17)
!38 = distinct !DISubprogram(name: "main", scope: !2, file: !2, line: 28, type: !39, scopeLine: 28, spFlags: DISPFlagDefinition, unit: !7)
!39 = !DISubroutineType(types: !40)
!40 = !{!20}
!41 = !DILocation(line: 29, column: 20, scope: !38)
!42 = !DILocation(line: 29, column: 5, scope: !38)
!43 = !DILocation(line: 30, column: 5, scope: !38)

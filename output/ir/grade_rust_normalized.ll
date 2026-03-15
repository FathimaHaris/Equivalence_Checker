; ModuleID = '/tmp/equivalence_checker/grade_rs_opt_display.bc'
source_filename = "grade_rust_harness.669e2647-cgu.0"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

@alloc_50e6ecab10eea712aa71caa980b8b020 = private unnamed_addr constant <{ [6 x i8] }> <{ [6 x i8] c"score\00" }>, align 1

; Function Attrs: nonlazybind uwtable
define i32 @_ZN18grade_rust_harness5grade17h605673be977c1f93E(i32 %score) unnamed_addr #0 {
start:
  %_2 = icmp sge i32 %score, 90
  br i1 %_2, label %bb1, label %bb2

bb2:                                              ; preds = %start
  %_3 = icmp sge i32 %score, 75
  br i1 %_3, label %bb3, label %bb4

bb1:                                              ; preds = %start
  br label %bb5

bb5:                                              ; preds = %bb3, %bb4, %bb1
  %.0 = phi i32 [ 4, %bb1 ], [ 3, %bb3 ], [ 1, %bb4 ]
  ret i32 %.0

bb4:                                              ; preds = %bb2
  br label %bb5

bb3:                                              ; preds = %bb2
  br label %bb5
}

; Function Attrs: nonlazybind uwtable
define i32 @klee_harness() unnamed_addr #0 {
start:
  %score = alloca i32, align 4
  store i32 0, ptr %score, align 4
  call void @klee_make_symbolic(ptr %score, i64 4, ptr @alloc_50e6ecab10eea712aa71caa980b8b020)
  %_14 = load i32, ptr %score, align 4, !noundef !2
  %_13 = icmp sge i32 %_14, 0
  br i1 %_13, label %bb5, label %bb4

bb4:                                              ; preds = %start
  br label %bb6

bb5:                                              ; preds = %start
  %_16 = load i32, ptr %score, align 4, !noundef !2
  %_15 = icmp sle i32 %_16, 100
  %0 = zext i1 %_15 to i8
  br label %bb6

bb6:                                              ; preds = %bb5, %bb4
  %_12.0 = phi i8 [ %0, %bb5 ], [ 0, %bb4 ]
  %1 = trunc i8 %_12.0 to i1
  %_11 = zext i1 %1 to i32
  call void @klee_assume(i32 %_11)
  %_17 = load i32, ptr %score, align 4, !noundef !2
  %2 = call i32 @_ZN18grade_rust_harness5grade17h605673be977c1f93E(i32 %_17)
  ret i32 %2
}

; Function Attrs: nonlazybind uwtable
declare void @klee_make_symbolic(ptr, i64, ptr) unnamed_addr #0

; Function Attrs: nonlazybind uwtable
declare void @klee_assume(i32) unnamed_addr #0

attributes #0 = { nonlazybind uwtable "probe-stack"="__rust_probestack" "target-cpu"="x86-64" }

!llvm.module.flags = !{!0, !1}

!0 = !{i32 7, !"PIC Level", i32 2}
!1 = !{i32 2, !"RtLibUseGOT", i32 1}
!2 = !{}

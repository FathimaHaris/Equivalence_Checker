; ModuleID = '/tmp/equivalence_checker/in_range_rust_normalized.bc'
source_filename = "in_range_rust_harness.a0019b2a-cgu.0"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

@alloc_c9c957c0c8511304e1f0e63463442336 = private unnamed_addr constant <{ [2 x i8] }> <{ [2 x i8] c"x\00" }>, align 1

; Function Attrs: nonlazybind uwtable
define internal i32 @_ZN21in_range_rust_harness8in_range17he535fbb57ba189ddE(i32 %x) unnamed_addr #0 {
start:
  %_3 = icmp sgt i32 %x, 0
  br i1 %_3, label %bb2, label %bb1

bb1:                                              ; preds = %start
  br label %bb3

bb2:                                              ; preds = %start
  %_4 = icmp sle i32 %x, 10
  %0 = zext i1 %_4 to i8
  br label %bb3

bb3:                                              ; preds = %bb2, %bb1
  %_2.0 = phi i8 [ %0, %bb2 ], [ 0, %bb1 ]
  %1 = trunc i8 %_2.0 to i1
  br i1 %1, label %bb4, label %bb5

bb5:                                              ; preds = %bb3
  br label %bb6

bb4:                                              ; preds = %bb3
  br label %bb6

bb6:                                              ; preds = %bb4, %bb5
  %.0 = phi i32 [ 1, %bb4 ], [ 0, %bb5 ]
  ret i32 %.0
}

; Function Attrs: nonlazybind uwtable
define i32 @klee_harness() unnamed_addr #0 {
start:
  %x = alloca i32, align 4
  store i32 0, ptr %x, align 4
  call void @klee_make_symbolic(ptr %x, i64 4, ptr @alloc_c9c957c0c8511304e1f0e63463442336)
  %_14 = load i32, ptr %x, align 4, !noundef !2
  %_13 = icmp sge i32 %_14, -5
  br i1 %_13, label %bb5, label %bb4

bb4:                                              ; preds = %start
  br label %bb6

bb5:                                              ; preds = %start
  %_16 = load i32, ptr %x, align 4, !noundef !2
  %_15 = icmp sle i32 %_16, 15
  %0 = zext i1 %_15 to i8
  br label %bb6

bb6:                                              ; preds = %bb5, %bb4
  %_12.0 = phi i8 [ %0, %bb5 ], [ 0, %bb4 ]
  %1 = trunc i8 %_12.0 to i1
  %_11 = zext i1 %1 to i32
  call void @klee_assume(i32 %_11)
  %_17 = load i32, ptr %x, align 4, !noundef !2
  %2 = call i32 @_ZN21in_range_rust_harness8in_range17he535fbb57ba189ddE(i32 %_17)
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

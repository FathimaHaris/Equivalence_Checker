; ModuleID = 'in_range_rust_harness.a0019b2a-cgu.0'
source_filename = "in_range_rust_harness.a0019b2a-cgu.0"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

@alloc_c9c957c0c8511304e1f0e63463442336 = private unnamed_addr constant <{ [2 x i8] }> <{ [2 x i8] c"x\00" }>, align 1

; in_range_rust_harness::in_range
; Function Attrs: nonlazybind uwtable
define internal i32 @_ZN21in_range_rust_harness8in_range17he535fbb57ba189ddE(i32 %x) unnamed_addr #0 {
start:
  %0 = alloca i32, align 4
  %_2 = icmp slt i32 %x, 0
  br i1 %_2, label %bb1, label %bb2

bb2:                                              ; preds = %start
  %_3 = icmp slt i32 %x, 10
  br i1 %_3, label %bb3, label %bb4

bb1:                                              ; preds = %start
  store i32 -1, ptr %0, align 4
  br label %bb5

bb5:                                              ; preds = %bb4, %bb3, %bb1
  %1 = load i32, ptr %0, align 4, !noundef !2
  ret i32 %1

bb4:                                              ; preds = %bb2
  store i32 2, ptr %0, align 4
  br label %bb5

bb3:                                              ; preds = %bb2
  store i32 1, ptr %0, align 4
  br label %bb5
}

; Function Attrs: nonlazybind uwtable
define i32 @klee_harness() unnamed_addr #0 {
start:
  %_12 = alloca i8, align 1
  %x = alloca i32, align 4
  store i32 0, ptr %x, align 4
  call void @klee_make_symbolic(ptr %x, i64 4, ptr @alloc_c9c957c0c8511304e1f0e63463442336)
  %_14 = load i32, ptr %x, align 4, !noundef !2
  %_13 = icmp sge i32 %_14, -5
  br i1 %_13, label %bb5, label %bb4

bb4:                                              ; preds = %start
  store i8 0, ptr %_12, align 1
  br label %bb6

bb5:                                              ; preds = %start
  %_16 = load i32, ptr %x, align 4, !noundef !2
  %_15 = icmp sle i32 %_16, 15
  %0 = zext i1 %_15 to i8
  store i8 %0, ptr %_12, align 1
  br label %bb6

bb6:                                              ; preds = %bb4, %bb5
  %1 = load i8, ptr %_12, align 1, !range !3, !noundef !2
  %2 = trunc i8 %1 to i1
  %_11 = zext i1 %2 to i32
  call void @klee_assume(i32 %_11)
  %_17 = load i32, ptr %x, align 4, !noundef !2
; call in_range_rust_harness::in_range
  %3 = call i32 @_ZN21in_range_rust_harness8in_range17he535fbb57ba189ddE(i32 %_17)
  ret i32 %3
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
!3 = !{i8 0, i8 2}

; ModuleID = '/tmp/equivalence_checker/classify_rust_normalized.bc'
source_filename = "classify_rust_harness.e9e7fb7b-cgu.0"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

@alloc_c9c957c0c8511304e1f0e63463442336 = private unnamed_addr constant <{ [2 x i8] }> <{ [2 x i8] c"x\00" }>, align 1

; Function Attrs: nonlazybind uwtable
define internal i32 @_ZN21classify_rust_harness8classify17h4cf0f8611e9f9f91E(i32 %x) unnamed_addr #0 {
start:
  %_2 = icmp slt i32 %x, 0
  br i1 %_2, label %bb1, label %bb2

bb2:                                              ; preds = %start
  %0 = icmp eq i32 %x, 0
  br i1 %0, label %bb3, label %bb4

bb1:                                              ; preds = %start
  br label %bb9

bb9:                                              ; preds = %bb7, %bb8, %bb5, %bb3, %bb1
  %.0 = phi i32 [ -1, %bb1 ], [ 0, %bb3 ], [ 1, %bb5 ], [ 2, %bb7 ], [ 3, %bb8 ]
  ret i32 %.0

bb3:                                              ; preds = %bb2
  br label %bb9

bb4:                                              ; preds = %bb2
  %_3 = icmp slt i32 %x, 10
  br i1 %_3, label %bb5, label %bb6

bb6:                                              ; preds = %bb4
  %_4 = icmp slt i32 %x, 100
  br i1 %_4, label %bb7, label %bb8

bb5:                                              ; preds = %bb4
  br label %bb9

bb8:                                              ; preds = %bb6
  br label %bb9

bb7:                                              ; preds = %bb6
  br label %bb9
}

; Function Attrs: nonlazybind uwtable
define i32 @klee_harness() unnamed_addr #0 {
start:
  %x = alloca i32, align 4
  store i32 0, ptr %x, align 4
  call void @klee_make_symbolic(ptr %x, i64 4, ptr @alloc_c9c957c0c8511304e1f0e63463442336)
  %_14 = load i32, ptr %x, align 4, !noundef !2
  %_13 = icmp sge i32 %_14, -10
  br i1 %_13, label %bb5, label %bb4

bb4:                                              ; preds = %start
  br label %bb6

bb5:                                              ; preds = %start
  %_16 = load i32, ptr %x, align 4, !noundef !2
  %_15 = icmp sle i32 %_16, 110
  %0 = zext i1 %_15 to i8
  br label %bb6

bb6:                                              ; preds = %bb5, %bb4
  %_12.0 = phi i8 [ %0, %bb5 ], [ 0, %bb4 ]
  %1 = trunc i8 %_12.0 to i1
  %_11 = zext i1 %1 to i32
  call void @klee_assume(i32 %_11)
  %_17 = load i32, ptr %x, align 4, !noundef !2
  %2 = call i32 @_ZN21classify_rust_harness8classify17h4cf0f8611e9f9f91E(i32 %_17)
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

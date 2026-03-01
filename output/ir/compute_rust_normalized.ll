; ModuleID = '/tmp/equivalence_checker/compute_rs_opt_display.bc'
source_filename = "compute_rust_harness.bce71cb4-cgu.0"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

@alloc_c9c957c0c8511304e1f0e63463442336 = private unnamed_addr constant <{ [2 x i8] }> <{ [2 x i8] c"x\00" }>, align 1
@alloc_95bd63817c298ea3373cf06db93d3c57 = private unnamed_addr constant <{ [2 x i8] }> <{ [2 x i8] c"y\00" }>, align 1

; Function Attrs: nonlazybind uwtable
define i32 @_ZN20compute_rust_harness7compute17h62be4aa0f57f4409E(i32 %x, i32 %y) unnamed_addr #0 {
start:
  %_3 = icmp sgt i32 %x, 10
  br i1 %_3, label %bb1, label %bb2

bb2:                                              ; preds = %start
  %0 = mul i32 %x, %y
  br label %bb3

bb1:                                              ; preds = %start
  %1 = add i32 %x, %y
  br label %bb3

bb3:                                              ; preds = %bb1, %bb2
  %.0 = phi i32 [ %1, %bb1 ], [ %0, %bb2 ]
  ret i32 %.0
}

; Function Attrs: nonlazybind uwtable
define i32 @klee_harness() unnamed_addr #0 {
start:
  %y = alloca i32, align 4
  %x = alloca i32, align 4
  store i32 0, ptr %x, align 4
  store i32 0, ptr %y, align 4
  call void @klee_make_symbolic(ptr %x, i64 4, ptr @alloc_c9c957c0c8511304e1f0e63463442336)
  call void @klee_make_symbolic(ptr %y, i64 4, ptr @alloc_95bd63817c298ea3373cf06db93d3c57)
  %_23 = load i32, ptr %x, align 4, !noundef !2
  %_22 = icmp sge i32 %_23, 0
  br i1 %_22, label %bb8, label %bb7

bb7:                                              ; preds = %start
  br label %bb9

bb8:                                              ; preds = %start
  %_25 = load i32, ptr %x, align 4, !noundef !2
  %_24 = icmp sle i32 %_25, 100
  %0 = zext i1 %_24 to i8
  br label %bb9

bb9:                                              ; preds = %bb8, %bb7
  %_21.0 = phi i8 [ %0, %bb8 ], [ 0, %bb7 ]
  %1 = trunc i8 %_21.0 to i1
  %_20 = zext i1 %1 to i32
  call void @klee_assume(i32 %_20)
  %_30 = load i32, ptr %y, align 4, !noundef !2
  %_29 = icmp sge i32 %_30, 0
  br i1 %_29, label %bb12, label %bb11

bb11:                                             ; preds = %bb9
  br label %bb13

bb12:                                             ; preds = %bb9
  %_32 = load i32, ptr %y, align 4, !noundef !2
  %_31 = icmp sle i32 %_32, 100
  %2 = zext i1 %_31 to i8
  br label %bb13

bb13:                                             ; preds = %bb12, %bb11
  %_28.0 = phi i8 [ %2, %bb12 ], [ 0, %bb11 ]
  %3 = trunc i8 %_28.0 to i1
  %_27 = zext i1 %3 to i32
  call void @klee_assume(i32 %_27)
  %_33 = load i32, ptr %x, align 4, !noundef !2
  %_34 = load i32, ptr %y, align 4, !noundef !2
  %4 = call i32 @_ZN20compute_rust_harness7compute17h62be4aa0f57f4409E(i32 %_33, i32 %_34)
  ret i32 %4
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

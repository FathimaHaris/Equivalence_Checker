; ModuleID = '/tmp/equivalence_checker/clamp_rs_opt_display.bc'
source_filename = "clamp_rust_harness.1a5dc188-cgu.0"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

@alloc_c9c957c0c8511304e1f0e63463442336 = private unnamed_addr constant <{ [2 x i8] }> <{ [2 x i8] c"x\00" }>, align 1
@alloc_68724ddb2a6d6897e12691e9bc7ec7f1 = private unnamed_addr constant <{ [3 x i8] }> <{ [3 x i8] c"lo\00" }>, align 1
@alloc_4c33640a8b80a8d3ca79b92a77ea3689 = private unnamed_addr constant <{ [3 x i8] }> <{ [3 x i8] c"hi\00" }>, align 1

; Function Attrs: nonlazybind uwtable
define i32 @_ZN18clamp_rust_harness5clamp17hb1d61b72a09b602eE(i32 %x, i32 %lo, i32 %hi) unnamed_addr #0 {
start:
  %_4 = icmp slt i32 %x, %lo
  br i1 %_4, label %bb1, label %bb2

bb2:                                              ; preds = %start
  %_5 = icmp sgt i32 %x, %hi
  br i1 %_5, label %bb3, label %bb4

bb1:                                              ; preds = %start
  br label %bb5

bb5:                                              ; preds = %bb3, %bb4, %bb1
  %.0 = phi i32 [ %lo, %bb1 ], [ %hi, %bb3 ], [ %x, %bb4 ]
  ret i32 %.0

bb4:                                              ; preds = %bb2
  br label %bb5

bb3:                                              ; preds = %bb2
  br label %bb5
}

; Function Attrs: nonlazybind uwtable
define i32 @klee_harness() unnamed_addr #0 {
start:
  %hi = alloca i32, align 4
  %lo = alloca i32, align 4
  %x = alloca i32, align 4
  store i32 0, ptr %x, align 4
  store i32 0, ptr %lo, align 4
  store i32 0, ptr %hi, align 4
  call void @klee_make_symbolic(ptr %x, i64 4, ptr @alloc_c9c957c0c8511304e1f0e63463442336)
  call void @klee_make_symbolic(ptr %lo, i64 4, ptr @alloc_68724ddb2a6d6897e12691e9bc7ec7f1)
  call void @klee_make_symbolic(ptr %hi, i64 4, ptr @alloc_4c33640a8b80a8d3ca79b92a77ea3689)
  %_32 = load i32, ptr %x, align 4, !noundef !2
  %_31 = icmp sge i32 %_32, 0
  br i1 %_31, label %bb11, label %bb10

bb10:                                             ; preds = %start
  br label %bb12

bb11:                                             ; preds = %start
  %_34 = load i32, ptr %x, align 4, !noundef !2
  %_33 = icmp sle i32 %_34, 100
  %0 = zext i1 %_33 to i8
  br label %bb12

bb12:                                             ; preds = %bb11, %bb10
  %_30.0 = phi i8 [ %0, %bb11 ], [ 0, %bb10 ]
  %1 = trunc i8 %_30.0 to i1
  %_29 = zext i1 %1 to i32
  call void @klee_assume(i32 %_29)
  %_39 = load i32, ptr %lo, align 4, !noundef !2
  %_38 = icmp sge i32 %_39, 0
  br i1 %_38, label %bb15, label %bb14

bb14:                                             ; preds = %bb12
  br label %bb16

bb15:                                             ; preds = %bb12
  %_41 = load i32, ptr %lo, align 4, !noundef !2
  %_40 = icmp sle i32 %_41, 100
  %2 = zext i1 %_40 to i8
  br label %bb16

bb16:                                             ; preds = %bb15, %bb14
  %_37.0 = phi i8 [ %2, %bb15 ], [ 0, %bb14 ]
  %3 = trunc i8 %_37.0 to i1
  %_36 = zext i1 %3 to i32
  call void @klee_assume(i32 %_36)
  %_46 = load i32, ptr %hi, align 4, !noundef !2
  %_45 = icmp sge i32 %_46, 0
  br i1 %_45, label %bb19, label %bb18

bb18:                                             ; preds = %bb16
  br label %bb20

bb19:                                             ; preds = %bb16
  %_48 = load i32, ptr %hi, align 4, !noundef !2
  %_47 = icmp sle i32 %_48, 100
  %4 = zext i1 %_47 to i8
  br label %bb20

bb20:                                             ; preds = %bb19, %bb18
  %_44.0 = phi i8 [ %4, %bb19 ], [ 0, %bb18 ]
  %5 = trunc i8 %_44.0 to i1
  %_43 = zext i1 %5 to i32
  call void @klee_assume(i32 %_43)
  %_49 = load i32, ptr %x, align 4, !noundef !2
  %_50 = load i32, ptr %lo, align 4, !noundef !2
  %_51 = load i32, ptr %hi, align 4, !noundef !2
  %6 = call i32 @_ZN18clamp_rust_harness5clamp17hb1d61b72a09b602eE(i32 %_49, i32 %_50, i32 %_51)
  ret i32 %6
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

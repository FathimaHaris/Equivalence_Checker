; ModuleID = 'clamp_rust_harness.1a5dc188-cgu.0'
source_filename = "clamp_rust_harness.1a5dc188-cgu.0"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

@alloc_c9c957c0c8511304e1f0e63463442336 = private unnamed_addr constant <{ [2 x i8] }> <{ [2 x i8] c"x\00" }>, align 1
@alloc_74f6776751c2c367269ea679666d544b = private unnamed_addr constant <{ [4 x i8] }> <{ [4 x i8] c"low\00" }>, align 1
@alloc_67a7f43c565e41cd11f80e7546fb9bc7 = private unnamed_addr constant <{ [5 x i8] }> <{ [5 x i8] c"high\00" }>, align 1

; clamp_rust_harness::clamp
; Function Attrs: nonlazybind uwtable
define internal i32 @_ZN18clamp_rust_harness5clamp17hb1d61b72a09b602eE(i32 %x, i32 %lo, i32 %hi) unnamed_addr #0 {
start:
  %0 = alloca i32, align 4
  %_4 = icmp sgt i32 %x, %hi
  br i1 %_4, label %bb1, label %bb2

bb2:                                              ; preds = %start
  %_5 = icmp slt i32 %x, %lo
  br i1 %_5, label %bb3, label %bb4

bb1:                                              ; preds = %start
  store i32 %hi, ptr %0, align 4
  br label %bb5

bb5:                                              ; preds = %bb4, %bb3, %bb1
  %1 = load i32, ptr %0, align 4, !noundef !2
  ret i32 %1

bb4:                                              ; preds = %bb2
  store i32 %x, ptr %0, align 4
  br label %bb5

bb3:                                              ; preds = %bb2
  store i32 %lo, ptr %0, align 4
  br label %bb5
}

; Function Attrs: nonlazybind uwtable
define i32 @klee_harness() unnamed_addr #0 {
start:
  %_44 = alloca i8, align 1
  %_37 = alloca i8, align 1
  %_30 = alloca i8, align 1
  %high = alloca i32, align 4
  %low = alloca i32, align 4
  %x = alloca i32, align 4
  store i32 0, ptr %x, align 4
  store i32 0, ptr %low, align 4
  store i32 0, ptr %high, align 4
  call void @klee_make_symbolic(ptr %x, i64 4, ptr @alloc_c9c957c0c8511304e1f0e63463442336)
  call void @klee_make_symbolic(ptr %low, i64 4, ptr @alloc_74f6776751c2c367269ea679666d544b)
  call void @klee_make_symbolic(ptr %high, i64 4, ptr @alloc_67a7f43c565e41cd11f80e7546fb9bc7)
  %_32 = load i32, ptr %x, align 4, !noundef !2
  %_31 = icmp sge i32 %_32, 0
  br i1 %_31, label %bb11, label %bb10

bb10:                                             ; preds = %start
  store i8 0, ptr %_30, align 1
  br label %bb12

bb11:                                             ; preds = %start
  %_34 = load i32, ptr %x, align 4, !noundef !2
  %_33 = icmp sle i32 %_34, 100
  %0 = zext i1 %_33 to i8
  store i8 %0, ptr %_30, align 1
  br label %bb12

bb12:                                             ; preds = %bb10, %bb11
  %1 = load i8, ptr %_30, align 1, !range !3, !noundef !2
  %2 = trunc i8 %1 to i1
  %_29 = zext i1 %2 to i32
  call void @klee_assume(i32 %_29)
  %_39 = load i32, ptr %low, align 4, !noundef !2
  %_38 = icmp sge i32 %_39, 0
  br i1 %_38, label %bb15, label %bb14

bb14:                                             ; preds = %bb12
  store i8 0, ptr %_37, align 1
  br label %bb16

bb15:                                             ; preds = %bb12
  %_41 = load i32, ptr %low, align 4, !noundef !2
  %_40 = icmp sle i32 %_41, 100
  %3 = zext i1 %_40 to i8
  store i8 %3, ptr %_37, align 1
  br label %bb16

bb16:                                             ; preds = %bb14, %bb15
  %4 = load i8, ptr %_37, align 1, !range !3, !noundef !2
  %5 = trunc i8 %4 to i1
  %_36 = zext i1 %5 to i32
  call void @klee_assume(i32 %_36)
  %_46 = load i32, ptr %high, align 4, !noundef !2
  %_45 = icmp sge i32 %_46, 0
  br i1 %_45, label %bb19, label %bb18

bb18:                                             ; preds = %bb16
  store i8 0, ptr %_44, align 1
  br label %bb20

bb19:                                             ; preds = %bb16
  %_48 = load i32, ptr %high, align 4, !noundef !2
  %_47 = icmp sle i32 %_48, 100
  %6 = zext i1 %_47 to i8
  store i8 %6, ptr %_44, align 1
  br label %bb20

bb20:                                             ; preds = %bb18, %bb19
  %7 = load i8, ptr %_44, align 1, !range !3, !noundef !2
  %8 = trunc i8 %7 to i1
  %_43 = zext i1 %8 to i32
  call void @klee_assume(i32 %_43)
  %_49 = load i32, ptr %x, align 4, !noundef !2
  %_50 = load i32, ptr %low, align 4, !noundef !2
  %_51 = load i32, ptr %high, align 4, !noundef !2
; call clamp_rust_harness::clamp
  %9 = call i32 @_ZN18clamp_rust_harness5clamp17hb1d61b72a09b602eE(i32 %_49, i32 %_50, i32 %_51)
  ret i32 %9
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

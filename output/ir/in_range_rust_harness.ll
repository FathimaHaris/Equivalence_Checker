; ModuleID = 'in_range_rust_harness.a0019b2a-cgu.0'
source_filename = "in_range_rust_harness.a0019b2a-cgu.0"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

@alloc_c9c957c0c8511304e1f0e63463442336 = private unnamed_addr constant <{ [2 x i8] }> <{ [2 x i8] c"x\00" }>, align 1
@alloc_68724ddb2a6d6897e12691e9bc7ec7f1 = private unnamed_addr constant <{ [3 x i8] }> <{ [3 x i8] c"lo\00" }>, align 1
@alloc_4c33640a8b80a8d3ca79b92a77ea3689 = private unnamed_addr constant <{ [3 x i8] }> <{ [3 x i8] c"hi\00" }>, align 1
@alloc_2b4bd59261e18c3ed2c493b3402b4e47 = private unnamed_addr constant <{ [7 x i8] }> <{ [7 x i8] c"result\00" }>, align 1

; in_range_rust_harness::in_range
; Function Attrs: nonlazybind uwtable
define i32 @_ZN21in_range_rust_harness8in_range17hb5b5d345d08b781aE(i32 %x, i32 %lo, i32 %hi) unnamed_addr #0 {
start:
  %_4 = alloca i8, align 1
  %0 = alloca i32, align 4
  %_5 = icmp sgt i32 %x, %lo
  br i1 %_5, label %bb2, label %bb1

bb1:                                              ; preds = %start
  store i8 0, ptr %_4, align 1
  br label %bb3

bb2:                                              ; preds = %start
  %_6 = icmp sle i32 %x, %hi
  %1 = zext i1 %_6 to i8
  store i8 %1, ptr %_4, align 1
  br label %bb3

bb3:                                              ; preds = %bb1, %bb2
  %2 = load i8, ptr %_4, align 1, !range !2, !noundef !3
  %3 = trunc i8 %2 to i1
  br i1 %3, label %bb4, label %bb5

bb5:                                              ; preds = %bb3
  store i32 0, ptr %0, align 4
  br label %bb6

bb4:                                              ; preds = %bb3
  store i32 1, ptr %0, align 4
  br label %bb6

bb6:                                              ; preds = %bb5, %bb4
  %4 = load i32, ptr %0, align 4, !noundef !3
  ret i32 %4
}

; Function Attrs: nonlazybind uwtable
define i32 @klee_harness() unnamed_addr #0 {
start:
  %_44 = alloca i8, align 1
  %_37 = alloca i8, align 1
  %_30 = alloca i8, align 1
  %hi = alloca i32, align 4
  %lo = alloca i32, align 4
  %x = alloca i32, align 4
  %__result = alloca i32, align 4
  store i32 0, ptr %x, align 4
  store i32 0, ptr %lo, align 4
  store i32 0, ptr %hi, align 4
  call void @klee_make_symbolic(ptr %x, i64 4, ptr @alloc_c9c957c0c8511304e1f0e63463442336)
  call void @klee_make_symbolic(ptr %lo, i64 4, ptr @alloc_68724ddb2a6d6897e12691e9bc7ec7f1)
  call void @klee_make_symbolic(ptr %hi, i64 4, ptr @alloc_4c33640a8b80a8d3ca79b92a77ea3689)
  %_32 = load i32, ptr %x, align 4, !noundef !3
  %_31 = icmp sge i32 %_32, 0
  br i1 %_31, label %bb11, label %bb10

bb10:                                             ; preds = %start
  store i8 0, ptr %_30, align 1
  br label %bb12

bb11:                                             ; preds = %start
  %_34 = load i32, ptr %x, align 4, !noundef !3
  %_33 = icmp sle i32 %_34, 100
  %0 = zext i1 %_33 to i8
  store i8 %0, ptr %_30, align 1
  br label %bb12

bb12:                                             ; preds = %bb10, %bb11
  %1 = load i8, ptr %_30, align 1, !range !2, !noundef !3
  %2 = trunc i8 %1 to i1
  %_29 = zext i1 %2 to i32
  call void @klee_assume(i32 %_29)
  %_39 = load i32, ptr %lo, align 4, !noundef !3
  %_38 = icmp sge i32 %_39, 0
  br i1 %_38, label %bb15, label %bb14

bb14:                                             ; preds = %bb12
  store i8 0, ptr %_37, align 1
  br label %bb16

bb15:                                             ; preds = %bb12
  %_41 = load i32, ptr %lo, align 4, !noundef !3
  %_40 = icmp sle i32 %_41, 100
  %3 = zext i1 %_40 to i8
  store i8 %3, ptr %_37, align 1
  br label %bb16

bb16:                                             ; preds = %bb14, %bb15
  %4 = load i8, ptr %_37, align 1, !range !2, !noundef !3
  %5 = trunc i8 %4 to i1
  %_36 = zext i1 %5 to i32
  call void @klee_assume(i32 %_36)
  %_46 = load i32, ptr %hi, align 4, !noundef !3
  %_45 = icmp sge i32 %_46, 0
  br i1 %_45, label %bb19, label %bb18

bb18:                                             ; preds = %bb16
  store i8 0, ptr %_44, align 1
  br label %bb20

bb19:                                             ; preds = %bb16
  %_48 = load i32, ptr %hi, align 4, !noundef !3
  %_47 = icmp sle i32 %_48, 100
  %6 = zext i1 %_47 to i8
  store i8 %6, ptr %_44, align 1
  br label %bb20

bb20:                                             ; preds = %bb18, %bb19
  %7 = load i8, ptr %_44, align 1, !range !2, !noundef !3
  %8 = trunc i8 %7 to i1
  %_43 = zext i1 %8 to i32
  call void @klee_assume(i32 %_43)
  store i32 0, ptr %__result, align 4
  call void @klee_make_symbolic(ptr %__result, i64 4, ptr @alloc_2b4bd59261e18c3ed2c493b3402b4e47)
  %_60 = load i32, ptr %__result, align 4, !noundef !3
  %_62 = load i32, ptr %x, align 4, !noundef !3
  %_63 = load i32, ptr %lo, align 4, !noundef !3
  %_64 = load i32, ptr %hi, align 4, !noundef !3
; call in_range_rust_harness::in_range
  %_61 = call i32 @_ZN21in_range_rust_harness8in_range17hb5b5d345d08b781aE(i32 %_62, i32 %_63, i32 %_64)
  %_59 = icmp eq i32 %_60, %_61
  %_58 = zext i1 %_59 to i32
  call void @klee_assume(i32 %_58)
  %9 = load i32, ptr %__result, align 4, !noundef !3
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
!2 = !{i8 0, i8 2}
!3 = !{}

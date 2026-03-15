; ModuleID = 'max_rust_harness.548218a3-cgu.0'
source_filename = "max_rust_harness.548218a3-cgu.0"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

@alloc_3825570913bed8d1542cb0922a51bd95 = private unnamed_addr constant <{ [2 x i8] }> <{ [2 x i8] c"a\00" }>, align 1
@alloc_d0e6abc3fdad902977b26dc7b6a8e735 = private unnamed_addr constant <{ [2 x i8] }> <{ [2 x i8] c"b\00" }>, align 1
@alloc_2b4bd59261e18c3ed2c493b3402b4e47 = private unnamed_addr constant <{ [7 x i8] }> <{ [7 x i8] c"result\00" }>, align 1

; max_rust_harness::max
; Function Attrs: nonlazybind uwtable
define i32 @_ZN16max_rust_harness3max17h84572449b1bb627eE(i32 %a, i32 %b) unnamed_addr #0 {
start:
  %0 = alloca i32, align 4
  %_3 = icmp sgt i32 %a, %b
  br i1 %_3, label %bb1, label %bb2

bb2:                                              ; preds = %start
  store i32 %b, ptr %0, align 4
  br label %bb3

bb1:                                              ; preds = %start
  store i32 %b, ptr %0, align 4
  br label %bb3

bb3:                                              ; preds = %bb2, %bb1
  %1 = load i32, ptr %0, align 4, !noundef !2
  ret i32 %1
}

; Function Attrs: nonlazybind uwtable
define i32 @klee_harness() unnamed_addr #0 {
start:
  %_28 = alloca i8, align 1
  %_21 = alloca i8, align 1
  %b = alloca i32, align 4
  %a = alloca i32, align 4
  %__result = alloca i32, align 4
  store i32 0, ptr %a, align 4
  store i32 0, ptr %b, align 4
  call void @klee_make_symbolic(ptr %a, i64 4, ptr @alloc_3825570913bed8d1542cb0922a51bd95)
  call void @klee_make_symbolic(ptr %b, i64 4, ptr @alloc_d0e6abc3fdad902977b26dc7b6a8e735)
  %_23 = load i32, ptr %a, align 4, !noundef !2
  %_22 = icmp sge i32 %_23, 0
  br i1 %_22, label %bb8, label %bb7

bb7:                                              ; preds = %start
  store i8 0, ptr %_21, align 1
  br label %bb9

bb8:                                              ; preds = %start
  %_25 = load i32, ptr %a, align 4, !noundef !2
  %_24 = icmp sle i32 %_25, 100
  %0 = zext i1 %_24 to i8
  store i8 %0, ptr %_21, align 1
  br label %bb9

bb9:                                              ; preds = %bb7, %bb8
  %1 = load i8, ptr %_21, align 1, !range !3, !noundef !2
  %2 = trunc i8 %1 to i1
  %_20 = zext i1 %2 to i32
  call void @klee_assume(i32 %_20)
  %_30 = load i32, ptr %b, align 4, !noundef !2
  %_29 = icmp sge i32 %_30, 0
  br i1 %_29, label %bb12, label %bb11

bb11:                                             ; preds = %bb9
  store i8 0, ptr %_28, align 1
  br label %bb13

bb12:                                             ; preds = %bb9
  %_32 = load i32, ptr %b, align 4, !noundef !2
  %_31 = icmp sle i32 %_32, 100
  %3 = zext i1 %_31 to i8
  store i8 %3, ptr %_28, align 1
  br label %bb13

bb13:                                             ; preds = %bb11, %bb12
  %4 = load i8, ptr %_28, align 1, !range !3, !noundef !2
  %5 = trunc i8 %4 to i1
  %_27 = zext i1 %5 to i32
  call void @klee_assume(i32 %_27)
  store i32 0, ptr %__result, align 4
  call void @klee_make_symbolic(ptr %__result, i64 4, ptr @alloc_2b4bd59261e18c3ed2c493b3402b4e47)
  %_44 = load i32, ptr %__result, align 4, !noundef !2
  %_46 = load i32, ptr %a, align 4, !noundef !2
  %_47 = load i32, ptr %b, align 4, !noundef !2
; call max_rust_harness::max
  %_45 = call i32 @_ZN16max_rust_harness3max17h84572449b1bb627eE(i32 %_46, i32 %_47)
  %_43 = icmp eq i32 %_44, %_45
  %_42 = zext i1 %_43 to i32
  call void @klee_assume(i32 %_42)
  %6 = load i32, ptr %__result, align 4, !noundef !2
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
!3 = !{i8 0, i8 2}

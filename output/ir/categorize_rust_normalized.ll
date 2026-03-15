; ModuleID = '/tmp/equivalence_checker/categorize_rs_opt_display.bc'
source_filename = "categorize_rust_harness.0688213c-cgu.0"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

@alloc_3825570913bed8d1542cb0922a51bd95 = private unnamed_addr constant <{ [2 x i8] }> <{ [2 x i8] c"a\00" }>, align 1
@alloc_d0e6abc3fdad902977b26dc7b6a8e735 = private unnamed_addr constant <{ [2 x i8] }> <{ [2 x i8] c"b\00" }>, align 1
@alloc_2b4bd59261e18c3ed2c493b3402b4e47 = private unnamed_addr constant <{ [7 x i8] }> <{ [7 x i8] c"result\00" }>, align 1

; Function Attrs: nonlazybind uwtable
define i32 @_ZN23categorize_rust_harness10categorize17h582d853008441f4eE(i32 %a, i32 %b) unnamed_addr #0 {
start:
  %_3 = icmp sgt i32 %a, %b
  br i1 %_3, label %bb1, label %bb4

bb4:                                              ; preds = %start
  %_5 = icmp sgt i32 %b, 10
  br i1 %_5, label %bb5, label %bb6

bb1:                                              ; preds = %start
  %_4 = icmp sgt i32 %a, 10
  br i1 %_4, label %bb2, label %bb3

bb3:                                              ; preds = %bb1
  br label %bb7

bb2:                                              ; preds = %bb1
  br label %bb7

bb7:                                              ; preds = %bb5, %bb6, %bb2, %bb3
  %.0 = phi i32 [ 3, %bb2 ], [ 2, %bb3 ], [ 1, %bb5 ], [ 0, %bb6 ]
  ret i32 %.0

bb6:                                              ; preds = %bb4
  br label %bb7

bb5:                                              ; preds = %bb4
  br label %bb7
}

; Function Attrs: nonlazybind uwtable
define i32 @klee_harness() unnamed_addr #0 {
start:
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
  br label %bb9

bb8:                                              ; preds = %start
  %_25 = load i32, ptr %a, align 4, !noundef !2
  %_24 = icmp sle i32 %_25, 100
  %0 = zext i1 %_24 to i8
  br label %bb9

bb9:                                              ; preds = %bb8, %bb7
  %_21.0 = phi i8 [ %0, %bb8 ], [ 0, %bb7 ]
  %1 = trunc i8 %_21.0 to i1
  %_20 = zext i1 %1 to i32
  call void @klee_assume(i32 %_20)
  %_30 = load i32, ptr %b, align 4, !noundef !2
  %_29 = icmp sge i32 %_30, 0
  br i1 %_29, label %bb12, label %bb11

bb11:                                             ; preds = %bb9
  br label %bb13

bb12:                                             ; preds = %bb9
  %_32 = load i32, ptr %b, align 4, !noundef !2
  %_31 = icmp sle i32 %_32, 100
  %2 = zext i1 %_31 to i8
  br label %bb13

bb13:                                             ; preds = %bb12, %bb11
  %_28.0 = phi i8 [ %2, %bb12 ], [ 0, %bb11 ]
  %3 = trunc i8 %_28.0 to i1
  %_27 = zext i1 %3 to i32
  call void @klee_assume(i32 %_27)
  store i32 0, ptr %__result, align 4
  call void @klee_make_symbolic(ptr %__result, i64 4, ptr @alloc_2b4bd59261e18c3ed2c493b3402b4e47)
  %_44 = load i32, ptr %__result, align 4, !noundef !2
  %_46 = load i32, ptr %a, align 4, !noundef !2
  %_47 = load i32, ptr %b, align 4, !noundef !2
  %_45 = call i32 @_ZN23categorize_rust_harness10categorize17h582d853008441f4eE(i32 %_46, i32 %_47)
  %_43 = icmp eq i32 %_44, %_45
  %_42 = zext i1 %_43 to i32
  call void @klee_assume(i32 %_42)
  %4 = load i32, ptr %__result, align 4, !noundef !2
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

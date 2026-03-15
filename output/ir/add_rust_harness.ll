; ModuleID = 'add_rust_harness.cf448f61-cgu.0'
source_filename = "add_rust_harness.cf448f61-cgu.0"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

@alloc_3825570913bed8d1542cb0922a51bd95 = private unnamed_addr constant <{ [2 x i8] }> <{ [2 x i8] c"a\00" }>, align 1
@alloc_d0e6abc3fdad902977b26dc7b6a8e735 = private unnamed_addr constant <{ [2 x i8] }> <{ [2 x i8] c"b\00" }>, align 1

; add_rust_harness::add
; Function Attrs: nonlazybind uwtable
define i32 @_ZN16add_rust_harness3add17hf21f225b8d854c0bE(i32 %a, i32 %b) unnamed_addr #0 {
start:
  %0 = add i32 %a, %b
  ret i32 %0
}

; Function Attrs: nonlazybind uwtable
define i32 @klee_harness() unnamed_addr #0 {
start:
  %_28 = alloca i8, align 1
  %_21 = alloca i8, align 1
  %b = alloca i32, align 4
  %a = alloca i32, align 4
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
  %_33 = load i32, ptr %a, align 4, !noundef !2
  %_34 = load i32, ptr %b, align 4, !noundef !2
; call add_rust_harness::add
  %6 = call i32 @_ZN16add_rust_harness3add17hf21f225b8d854c0bE(i32 %_33, i32 %_34)
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

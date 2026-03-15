; ModuleID = '/tmp/equivalence_checker/factorial_rs_opt_display.bc'
source_filename = "factorial_rust_harness.cb3d6dad-cgu.0"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

@alloc_e01bdcd616f29df38e098e75c85b494d = private unnamed_addr constant <{ [2 x i8] }> <{ [2 x i8] c"n\00" }>, align 1
@alloc_2b4bd59261e18c3ed2c493b3402b4e47 = private unnamed_addr constant <{ [7 x i8] }> <{ [7 x i8] c"result\00" }>, align 1

; Function Attrs: nonlazybind uwtable
define i32 @_ZN22factorial_rust_harness9factorial17ha91f8b17fa47f0e8E(i32 %n) unnamed_addr #0 {
start:
  br label %bb1

bb1:                                              ; preds = %bb2, %start
  %result.0 = phi i32 [ 1, %start ], [ %0, %bb2 ]
  %i.0 = phi i32 [ 1, %start ], [ %1, %bb2 ]
  %_3 = icmp sle i32 %i.0, %n
  br i1 %_3, label %bb2, label %bb3

bb3:                                              ; preds = %bb1
  ret i32 %result.0

bb2:                                              ; preds = %bb1
  %0 = mul i32 %result.0, %i.0
  %1 = add i32 %i.0, 1
  br label %bb1
}

; Function Attrs: nonlazybind uwtable
define i32 @klee_harness() unnamed_addr #0 {
start:
  %n = alloca i32, align 4
  %__result = alloca i32, align 4
  store i32 0, ptr %n, align 4
  call void @klee_make_symbolic(ptr %n, i64 4, ptr @alloc_e01bdcd616f29df38e098e75c85b494d)
  %_14 = load i32, ptr %n, align 4, !noundef !2
  %_13 = icmp sge i32 %_14, 0
  br i1 %_13, label %bb5, label %bb4

bb4:                                              ; preds = %start
  br label %bb6

bb5:                                              ; preds = %start
  %_16 = load i32, ptr %n, align 4, !noundef !2
  %_15 = icmp sle i32 %_16, 100
  %0 = zext i1 %_15 to i8
  br label %bb6

bb6:                                              ; preds = %bb5, %bb4
  %_12.0 = phi i8 [ %0, %bb5 ], [ 0, %bb4 ]
  %1 = trunc i8 %_12.0 to i1
  %_11 = zext i1 %1 to i32
  call void @klee_assume(i32 %_11)
  store i32 0, ptr %__result, align 4
  call void @klee_make_symbolic(ptr %__result, i64 4, ptr @alloc_2b4bd59261e18c3ed2c493b3402b4e47)
  %_28 = load i32, ptr %__result, align 4, !noundef !2
  %_30 = load i32, ptr %n, align 4, !noundef !2
  %_29 = call i32 @_ZN22factorial_rust_harness9factorial17ha91f8b17fa47f0e8E(i32 %_30)
  %_27 = icmp eq i32 %_28, %_29
  %_26 = zext i1 %_27 to i32
  call void @klee_assume(i32 %_26)
  %2 = load i32, ptr %__result, align 4, !noundef !2
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

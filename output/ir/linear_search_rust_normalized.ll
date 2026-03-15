; ModuleID = '/tmp/equivalence_checker/linear_search_rs_opt_display.bc'
source_filename = "linear_search_rust_harness.9f71f5de-cgu.0"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

@alloc_57a221e6cd22e08c45e53aaad23a5763 = private unnamed_addr constant <{ [54 x i8] }> <{ [54 x i8] c"/tmp/equivalence_checker/linear_search_rust_harness.rs" }>, align 1
@alloc_5161b979cb4a7acfef15485fa4fe0419 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_57a221e6cd22e08c45e53aaad23a5763, [16 x i8] c"6\00\00\00\00\00\00\00\17\00\00\00\0C\00\00\00" }>, align 8
@alloc_526d5ced4cdc7443191f7459e7707f99 = private unnamed_addr constant <{ [7 x i8] }> <{ [7 x i8] c"target\00" }>, align 1
@alloc_2b4bd59261e18c3ed2c493b3402b4e47 = private unnamed_addr constant <{ [7 x i8] }> <{ [7 x i8] c"result\00" }>, align 1

; Function Attrs: nonlazybind uwtable
define i32 @_ZN26linear_search_rust_harness13linear_search17h88c9742cdc24eb3dE(i32 %target) unnamed_addr #0 {
start:
  %arr = alloca [6 x i32], align 4
  %0 = getelementptr inbounds [6 x i32], ptr %arr, i64 0, i64 0
  call void @llvm.memset.p0.i64(ptr align 4 %0, i8 0, i64 24, i1 false)
  %1 = getelementptr inbounds [6 x i32], ptr %arr, i64 0, i64 0
  store i32 3, ptr %1, align 4
  %2 = getelementptr inbounds [6 x i32], ptr %arr, i64 0, i64 1
  store i32 7, ptr %2, align 4
  %3 = getelementptr inbounds [6 x i32], ptr %arr, i64 0, i64 2
  store i32 1, ptr %3, align 4
  %4 = getelementptr inbounds [6 x i32], ptr %arr, i64 0, i64 3
  store i32 9, ptr %4, align 4
  %5 = getelementptr inbounds [6 x i32], ptr %arr, i64 0, i64 4
  store i32 4, ptr %5, align 4
  %6 = getelementptr inbounds [6 x i32], ptr %arr, i64 0, i64 5
  store i32 6, ptr %6, align 4
  br label %bb1

bb1:                                              ; preds = %bb5, %start
  %i.0 = phi i32 [ 0, %start ], [ %9, %bb5 ]
  %_10 = icmp slt i32 %i.0, 6
  br i1 %_10, label %bb2, label %bb6

bb6:                                              ; preds = %bb4, %bb1
  %found.0 = phi i32 [ %i.0, %bb4 ], [ -1, %bb1 ]
  ret i32 %found.0

bb2:                                              ; preds = %bb1
  %_14 = sext i32 %i.0 to i64
  %_17 = icmp ult i64 %_14, 6
  %7 = call i1 @llvm.expect.i1(i1 %_17, i1 true)
  br i1 %7, label %bb3, label %panic

bb3:                                              ; preds = %bb2
  %8 = getelementptr inbounds [6 x i32], ptr %arr, i64 0, i64 %_14
  %_13 = load i32, ptr %8, align 4, !noundef !2
  %_12 = icmp eq i32 %_13, %target
  br i1 %_12, label %bb4, label %bb5

panic:                                            ; preds = %bb2
  call void @_ZN4core9panicking18panic_bounds_check17haf06fefb23eba82dE(i64 %_14, i64 6, ptr align 8 @alloc_5161b979cb4a7acfef15485fa4fe0419) #4
  unreachable

bb5:                                              ; preds = %bb3
  %9 = add i32 %i.0, 1
  br label %bb1

bb4:                                              ; preds = %bb3
  br label %bb6
}

; Function Attrs: nonlazybind uwtable
define i32 @klee_harness() unnamed_addr #0 {
start:
  %target = alloca i32, align 4
  %__result = alloca i32, align 4
  store i32 0, ptr %target, align 4
  call void @klee_make_symbolic(ptr %target, i64 4, ptr @alloc_526d5ced4cdc7443191f7459e7707f99)
  %_14 = load i32, ptr %target, align 4, !noundef !2
  %_13 = icmp sge i32 %_14, 0
  br i1 %_13, label %bb5, label %bb4

bb4:                                              ; preds = %start
  br label %bb6

bb5:                                              ; preds = %start
  %_16 = load i32, ptr %target, align 4, !noundef !2
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
  %_30 = load i32, ptr %target, align 4, !noundef !2
  %_29 = call i32 @_ZN26linear_search_rust_harness13linear_search17h88c9742cdc24eb3dE(i32 %_30)
  %_27 = icmp eq i32 %_28, %_29
  %_26 = zext i1 %_27 to i32
  call void @klee_assume(i32 %_26)
  %2 = load i32, ptr %__result, align 4, !noundef !2
  ret i32 %2
}

; Function Attrs: argmemonly nocallback nofree nounwind willreturn writeonly
declare void @llvm.memset.p0.i64(ptr nocapture writeonly, i8, i64, i1 immarg) #1

; Function Attrs: nocallback nofree nosync nounwind readnone willreturn
declare i1 @llvm.expect.i1(i1, i1) #2

; Function Attrs: cold noinline noreturn nonlazybind uwtable
declare void @_ZN4core9panicking18panic_bounds_check17haf06fefb23eba82dE(i64, i64, ptr align 8) unnamed_addr #3

; Function Attrs: nonlazybind uwtable
declare void @klee_make_symbolic(ptr, i64, ptr) unnamed_addr #0

; Function Attrs: nonlazybind uwtable
declare void @klee_assume(i32) unnamed_addr #0

attributes #0 = { nonlazybind uwtable "probe-stack"="__rust_probestack" "target-cpu"="x86-64" }
attributes #1 = { argmemonly nocallback nofree nounwind willreturn writeonly }
attributes #2 = { nocallback nofree nosync nounwind readnone willreturn }
attributes #3 = { cold noinline noreturn nonlazybind uwtable "probe-stack"="__rust_probestack" "target-cpu"="x86-64" }
attributes #4 = { noreturn }

!llvm.module.flags = !{!0, !1}

!0 = !{i32 7, !"PIC Level", i32 2}
!1 = !{i32 2, !"RtLibUseGOT", i32 1}
!2 = !{}

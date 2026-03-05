; ModuleID = '/tmp/equivalence_checker/sum_to_n_rs_opt_display.bc'
source_filename = "sum_to_n_rust_harness.905a04fd-cgu.0"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

@alloc_e01bdcd616f29df38e098e75c85b494d = private unnamed_addr constant <{ [2 x i8] }> <{ [2 x i8] c"n\00" }>, align 1

; Function Attrs: inlinehint nonlazybind uwtable
define internal i32 @"_ZN47_$LT$i32$u20$as$u20$core..iter..range..Step$GT$17forward_unchecked17h76d5eb1cd768b405E"(i32 %start1, i64 %n) unnamed_addr #0 {
start:
  %rhs = trunc i64 %n to i32
  %0 = add nsw i32 %start1, %rhs
  ret i32 %0
}

; Function Attrs: inlinehint nonlazybind uwtable
define internal zeroext i1 @"_ZN4core3cmp5impls55_$LT$impl$u20$core..cmp..PartialOrd$u20$for$u20$i32$GT$2lt17ha9a0a097584d2d9dE"(ptr align 4 %self, ptr align 4 %other) unnamed_addr #0 {
start:
  %_3 = load i32, ptr %self, align 4, !noundef !2
  %_4 = load i32, ptr %other, align 4, !noundef !2
  %0 = icmp slt i32 %_3, %_4
  ret i1 %0
}

; Function Attrs: inlinehint nonlazybind uwtable
define i32 @_ZN4core3mem7replace17h4c93fda9c0250f6eE(ptr align 4 %dest, i32 %src) unnamed_addr #0 personality ptr @rust_eh_personality {
start:
  %0 = alloca { ptr, i32 }, align 8
  %tmp = alloca i32, align 4
  %src1 = alloca i32, align 4
  call void @llvm.memcpy.p0.p0.i64(ptr align 4 %tmp, ptr align 4 %dest, i64 4, i1 false)
  %self = load i32, ptr %tmp, align 4
  br label %bb4

bb4:                                              ; preds = %start
  store i32 %src, ptr %src1, align 4
  call void @llvm.memcpy.p0.p0.i64(ptr align 4 %dest, ptr align 4 %src1, i64 4, i1 false)
  ret i32 %self

bb3:                                              ; No predecessors!
  br i1 true, label %bb2, label %bb1

bb1:                                              ; preds = %bb2, %bb3
  %1 = load ptr, ptr %0, align 8, !noundef !2
  %2 = getelementptr inbounds { ptr, i32 }, ptr %0, i32 0, i32 1
  %3 = load i32, ptr %2, align 8, !noundef !2
  %4 = insertvalue { ptr, i32 } undef, ptr %1, 0
  %5 = insertvalue { ptr, i32 } %4, i32 %3, 1
  resume { ptr, i32 } %5

bb2:                                              ; preds = %bb3
  br label %bb1
}

; Function Attrs: inlinehint nonlazybind uwtable
define { i32, i32 } @"_ZN4core4iter5range101_$LT$impl$u20$core..iter..traits..iterator..Iterator$u20$for$u20$core..ops..range..Range$LT$A$GT$$GT$4next17hd251e5ff54faa7eaE"(ptr align 4 %self) unnamed_addr #0 {
start:
  %0 = call { i32, i32 } @"_ZN89_$LT$core..ops..range..Range$LT$T$GT$$u20$as$u20$core..iter..range..RangeIteratorImpl$GT$9spec_next17h2ae6c92106e33fa9E"(ptr align 4 %self)
  %1 = extractvalue { i32, i32 } %0, 0
  %2 = extractvalue { i32, i32 } %0, 1
  %3 = insertvalue { i32, i32 } undef, i32 %1, 0
  %4 = insertvalue { i32, i32 } %3, i32 %2, 1
  ret { i32, i32 } %4
}

; Function Attrs: inlinehint nonlazybind uwtable
define { i32, i32 } @"_ZN63_$LT$I$u20$as$u20$core..iter..traits..collect..IntoIterator$GT$9into_iter17he73e9f314a209d8cE"(i32 %self.0, i32 %self.1) unnamed_addr #0 {
start:
  %0 = insertvalue { i32, i32 } undef, i32 %self.0, 0
  %1 = insertvalue { i32, i32 } %0, i32 %self.1, 1
  ret { i32, i32 } %1
}

; Function Attrs: inlinehint nonlazybind uwtable
define { i32, i32 } @"_ZN89_$LT$core..ops..range..Range$LT$T$GT$$u20$as$u20$core..iter..range..RangeIteratorImpl$GT$9spec_next17h2ae6c92106e33fa9E"(ptr align 4 %self) unnamed_addr #0 {
start:
  %0 = alloca { i32, i32 }, align 4
  %_4 = getelementptr inbounds { i32, i32 }, ptr %self, i32 0, i32 1
  %_2 = call zeroext i1 @"_ZN4core3cmp5impls55_$LT$impl$u20$core..cmp..PartialOrd$u20$for$u20$i32$GT$2lt17ha9a0a097584d2d9dE"(ptr align 4 %self, ptr align 4 %_4)
  br i1 %_2, label %bb2, label %bb6

bb6:                                              ; preds = %start
  store i32 0, ptr %0, align 4
  br label %bb7

bb2:                                              ; preds = %start
  %1 = load i32, ptr %self, align 4, !noundef !2
  %n = call i32 @"_ZN47_$LT$i32$u20$as$u20$core..iter..range..Step$GT$17forward_unchecked17h76d5eb1cd768b405E"(i32 %1, i64 1)
  %_8 = call i32 @_ZN4core3mem7replace17h4c93fda9c0250f6eE(ptr align 4 %self, i32 %n)
  %2 = getelementptr inbounds { i32, i32 }, ptr %0, i32 0, i32 1
  store i32 %_8, ptr %2, align 4
  store i32 1, ptr %0, align 4
  br label %bb7

bb7:                                              ; preds = %bb2, %bb6
  %3 = getelementptr inbounds { i32, i32 }, ptr %0, i32 0, i32 0
  %4 = load i32, ptr %3, align 4, !range !3, !noundef !2
  %5 = getelementptr inbounds { i32, i32 }, ptr %0, i32 0, i32 1
  %6 = load i32, ptr %5, align 4
  %7 = insertvalue { i32, i32 } undef, i32 %4, 0
  %8 = insertvalue { i32, i32 } %7, i32 %6, 1
  ret { i32, i32 } %8
}

; Function Attrs: nonlazybind uwtable
define i32 @_ZN21sum_to_n_rust_harness8sum_to_n17hceb69fd3e3607e9bE(i32 %n) unnamed_addr #1 {
start:
  %_5 = alloca { i32, i32 }, align 4
  %iter = alloca { i32, i32 }, align 4
  %_3 = alloca { i32, i32 }, align 4
  store i32 0, ptr %_3, align 4
  %0 = getelementptr inbounds { i32, i32 }, ptr %_3, i32 0, i32 1
  store i32 %n, ptr %0, align 4
  %1 = getelementptr inbounds { i32, i32 }, ptr %_3, i32 0, i32 0
  %2 = load i32, ptr %1, align 4, !noundef !2
  %3 = getelementptr inbounds { i32, i32 }, ptr %_3, i32 0, i32 1
  %4 = load i32, ptr %3, align 4, !noundef !2
  %5 = call { i32, i32 } @"_ZN63_$LT$I$u20$as$u20$core..iter..traits..collect..IntoIterator$GT$9into_iter17he73e9f314a209d8cE"(i32 %2, i32 %4)
  %_2.0 = extractvalue { i32, i32 } %5, 0
  %_2.1 = extractvalue { i32, i32 } %5, 1
  %6 = getelementptr inbounds { i32, i32 }, ptr %iter, i32 0, i32 0
  store i32 %_2.0, ptr %6, align 4
  %7 = getelementptr inbounds { i32, i32 }, ptr %iter, i32 0, i32 1
  store i32 %_2.1, ptr %7, align 4
  br label %bb2

bb2:                                              ; preds = %bb4, %start
  %s.0 = phi i32 [ 0, %start ], [ %12, %bb4 ]
  %8 = call { i32, i32 } @"_ZN4core4iter5range101_$LT$impl$u20$core..iter..traits..iterator..Iterator$u20$for$u20$core..ops..range..Range$LT$A$GT$$GT$4next17hd251e5ff54faa7eaE"(ptr align 4 %iter)
  store { i32, i32 } %8, ptr %_5, align 4
  %9 = load i32, ptr %_5, align 4, !range !3, !noundef !2
  %_7 = zext i32 %9 to i64
  %10 = icmp eq i64 %_7, 0
  br i1 %10, label %bb6, label %bb4

bb6:                                              ; preds = %bb2
  ret i32 %s.0

bb4:                                              ; preds = %bb2
  %11 = getelementptr inbounds { i32, i32 }, ptr %_5, i32 0, i32 1
  %i = load i32, ptr %11, align 4, !noundef !2
  %12 = add i32 %s.0, %i
  br label %bb2

bb5:                                              ; No predecessors!
  unreachable
}

; Function Attrs: nonlazybind uwtable
define i32 @klee_harness() unnamed_addr #1 {
start:
  %n = alloca i32, align 4
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
  %_17 = load i32, ptr %n, align 4, !noundef !2
  %2 = call i32 @_ZN21sum_to_n_rust_harness8sum_to_n17hceb69fd3e3607e9bE(i32 %_17)
  ret i32 %2
}

; Function Attrs: nonlazybind uwtable
declare i32 @rust_eh_personality(i32, i32, i64, ptr, ptr) unnamed_addr #1

; Function Attrs: argmemonly nocallback nofree nounwind willreturn
declare void @llvm.memcpy.p0.p0.i64(ptr noalias nocapture writeonly, ptr noalias nocapture readonly, i64, i1 immarg) #2

; Function Attrs: nonlazybind uwtable
declare void @klee_make_symbolic(ptr, i64, ptr) unnamed_addr #1

; Function Attrs: nonlazybind uwtable
declare void @klee_assume(i32) unnamed_addr #1

attributes #0 = { inlinehint nonlazybind uwtable "probe-stack"="__rust_probestack" "target-cpu"="x86-64" }
attributes #1 = { nonlazybind uwtable "probe-stack"="__rust_probestack" "target-cpu"="x86-64" }
attributes #2 = { argmemonly nocallback nofree nounwind willreturn }

!llvm.module.flags = !{!0, !1}

!0 = !{i32 7, !"PIC Level", i32 2}
!1 = !{i32 2, !"RtLibUseGOT", i32 1}
!2 = !{}
!3 = !{i32 0, i32 2}

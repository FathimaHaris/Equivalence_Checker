; ModuleID = '/tmp/equivalence_checker/count_even_rs_opt_display.bc'
source_filename = "count_even_rust_harness.e7a58843-cgu.0"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

%"core::ops::range::RangeInclusive<i32>" = type { i32, i32, i8, [3 x i8] }

@alloc_e01bdcd616f29df38e098e75c85b494d = private unnamed_addr constant <{ [2 x i8] }> <{ [2 x i8] c"n\00" }>, align 1

; Function Attrs: inlinehint nonlazybind uwtable
define { i32, i32 } @"_ZN107_$LT$core..ops..range..RangeInclusive$LT$T$GT$$u20$as$u20$core..iter..range..RangeInclusiveIteratorImpl$GT$9spec_next17hff740fc2eaf9c5afE"(ptr align 4 %self) unnamed_addr #0 {
start:
  %0 = alloca { i32, i32 }, align 4
  %1 = getelementptr inbounds %"core::ops::range::RangeInclusive<i32>", ptr %self, i32 0, i32 2
  %2 = load i8, ptr %1, align 4, !range !2, !noundef !3
  %_15 = trunc i8 %2 to i1
  br i1 %_15, label %bb12, label %bb13

bb13:                                             ; preds = %start
  %_19 = getelementptr inbounds %"core::ops::range::RangeInclusive<i32>", ptr %self, i32 0, i32 1
  %_17 = call zeroext i1 @"_ZN4core3cmp5impls55_$LT$impl$u20$core..cmp..PartialOrd$u20$for$u20$i32$GT$2le17h43ba352206d19e17E"(ptr align 4 %self, ptr align 4 %_19)
  %_16 = xor i1 %_17, true
  %3 = zext i1 %_16 to i8
  br label %bb14

bb12:                                             ; preds = %start
  br label %bb14

bb14:                                             ; preds = %bb12, %bb13
  %_2.0 = phi i8 [ 1, %bb12 ], [ %3, %bb13 ]
  %4 = trunc i8 %_2.0 to i1
  br i1 %4, label %bb1, label %bb2

bb2:                                              ; preds = %bb14
  %_6 = getelementptr inbounds %"core::ops::range::RangeInclusive<i32>", ptr %self, i32 0, i32 1
  %is_iterating = call zeroext i1 @"_ZN4core3cmp5impls55_$LT$impl$u20$core..cmp..PartialOrd$u20$for$u20$i32$GT$2lt17hda650730b4b8d38dE"(ptr align 4 %self, ptr align 4 %_6)
  br i1 %is_iterating, label %bb4, label %bb8

bb1:                                              ; preds = %bb14
  store i32 0, ptr %0, align 4
  br label %bb11

bb11:                                             ; preds = %bb10, %bb1
  %5 = getelementptr inbounds { i32, i32 }, ptr %0, i32 0, i32 0
  %6 = load i32, ptr %5, align 4, !range !4, !noundef !3
  %7 = getelementptr inbounds { i32, i32 }, ptr %0, i32 0, i32 1
  %8 = load i32, ptr %7, align 4
  %9 = insertvalue { i32, i32 } undef, i32 %6, 0
  %10 = insertvalue { i32, i32 } %9, i32 %8, 1
  ret { i32, i32 } %10

bb8:                                              ; preds = %bb2
  %11 = getelementptr inbounds %"core::ops::range::RangeInclusive<i32>", ptr %self, i32 0, i32 2
  store i8 1, ptr %11, align 4
  %12 = load i32, ptr %self, align 4, !noundef !3
  br label %bb10

bb4:                                              ; preds = %bb2
  %13 = load i32, ptr %self, align 4, !noundef !3
  %n = call i32 @"_ZN47_$LT$i32$u20$as$u20$core..iter..range..Step$GT$17forward_unchecked17h96bcf13761f4e92fE"(i32 %13, i64 1)
  %14 = call i32 @_ZN4core3mem7replace17hd1d334675b2c94ebE(ptr align 4 %self, i32 %n)
  br label %bb10

bb10:                                             ; preds = %bb4, %bb8
  %_7.0 = phi i32 [ %14, %bb4 ], [ %12, %bb8 ]
  %15 = getelementptr inbounds { i32, i32 }, ptr %0, i32 0, i32 1
  store i32 %_7.0, ptr %15, align 4
  store i32 1, ptr %0, align 4
  br label %bb11
}

; Function Attrs: inlinehint nonlazybind uwtable
define internal i32 @"_ZN47_$LT$i32$u20$as$u20$core..iter..range..Step$GT$17forward_unchecked17h96bcf13761f4e92fE"(i32 %start1, i64 %n) unnamed_addr #0 {
start:
  %rhs = trunc i64 %n to i32
  %0 = add nsw i32 %start1, %rhs
  ret i32 %0
}

; Function Attrs: inlinehint nonlazybind uwtable
define internal zeroext i1 @"_ZN4core3cmp5impls55_$LT$impl$u20$core..cmp..PartialOrd$u20$for$u20$i32$GT$2le17h43ba352206d19e17E"(ptr align 4 %self, ptr align 4 %other) unnamed_addr #0 {
start:
  %_3 = load i32, ptr %self, align 4, !noundef !3
  %_4 = load i32, ptr %other, align 4, !noundef !3
  %0 = icmp sle i32 %_3, %_4
  ret i1 %0
}

; Function Attrs: inlinehint nonlazybind uwtable
define internal zeroext i1 @"_ZN4core3cmp5impls55_$LT$impl$u20$core..cmp..PartialOrd$u20$for$u20$i32$GT$2lt17hda650730b4b8d38dE"(ptr align 4 %self, ptr align 4 %other) unnamed_addr #0 {
start:
  %_3 = load i32, ptr %self, align 4, !noundef !3
  %_4 = load i32, ptr %other, align 4, !noundef !3
  %0 = icmp slt i32 %_3, %_4
  ret i1 %0
}

; Function Attrs: inlinehint nonlazybind uwtable
define i32 @_ZN4core3mem7replace17hd1d334675b2c94ebE(ptr align 4 %dest, i32 %src) unnamed_addr #0 personality ptr @rust_eh_personality {
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
  %1 = load ptr, ptr %0, align 8, !noundef !3
  %2 = getelementptr inbounds { ptr, i32 }, ptr %0, i32 0, i32 1
  %3 = load i32, ptr %2, align 8, !noundef !3
  %4 = insertvalue { ptr, i32 } undef, ptr %1, 0
  %5 = insertvalue { ptr, i32 } %4, i32 %3, 1
  resume { ptr, i32 } %5

bb2:                                              ; preds = %bb3
  br label %bb1
}

; Function Attrs: inlinehint nonlazybind uwtable
define void @"_ZN4core3ops5range25RangeInclusive$LT$Idx$GT$3new17ha2d3ac532bfe095cE"(ptr sret(%"core::ops::range::RangeInclusive<i32>") %0, i32 %start1, i32 %end) unnamed_addr #0 {
start:
  store i32 %start1, ptr %0, align 4
  %1 = getelementptr inbounds %"core::ops::range::RangeInclusive<i32>", ptr %0, i32 0, i32 1
  store i32 %end, ptr %1, align 4
  %2 = getelementptr inbounds %"core::ops::range::RangeInclusive<i32>", ptr %0, i32 0, i32 2
  store i8 0, ptr %2, align 4
  ret void
}

; Function Attrs: inlinehint nonlazybind uwtable
define { i32, i32 } @"_ZN4core4iter5range110_$LT$impl$u20$core..iter..traits..iterator..Iterator$u20$for$u20$core..ops..range..RangeInclusive$LT$A$GT$$GT$4next17h9ea9c0cd3a033b22E"(ptr align 4 %self) unnamed_addr #0 {
start:
  %0 = call { i32, i32 } @"_ZN107_$LT$core..ops..range..RangeInclusive$LT$T$GT$$u20$as$u20$core..iter..range..RangeInclusiveIteratorImpl$GT$9spec_next17hff740fc2eaf9c5afE"(ptr align 4 %self)
  %1 = extractvalue { i32, i32 } %0, 0
  %2 = extractvalue { i32, i32 } %0, 1
  %3 = insertvalue { i32, i32 } undef, i32 %1, 0
  %4 = insertvalue { i32, i32 } %3, i32 %2, 1
  ret { i32, i32 } %4
}

; Function Attrs: inlinehint nonlazybind uwtable
define void @"_ZN63_$LT$I$u20$as$u20$core..iter..traits..collect..IntoIterator$GT$9into_iter17h61da204698638269E"(ptr sret(%"core::ops::range::RangeInclusive<i32>") %0, ptr %self) unnamed_addr #0 {
start:
  call void @llvm.memcpy.p0.p0.i64(ptr align 4 %0, ptr align 4 %self, i64 12, i1 false)
  ret void
}

; Function Attrs: nonlazybind uwtable
define i32 @_ZN23count_even_rust_harness10count_even17h9dfdce1085948c80E(i32 %n) unnamed_addr #1 {
start:
  %_5 = alloca { i32, i32 }, align 4
  %iter = alloca %"core::ops::range::RangeInclusive<i32>", align 4
  %_3 = alloca %"core::ops::range::RangeInclusive<i32>", align 4
  %_2 = alloca %"core::ops::range::RangeInclusive<i32>", align 4
  call void @"_ZN4core3ops5range25RangeInclusive$LT$Idx$GT$3new17ha2d3ac532bfe095cE"(ptr sret(%"core::ops::range::RangeInclusive<i32>") %_3, i32 0, i32 %n)
  call void @"_ZN63_$LT$I$u20$as$u20$core..iter..traits..collect..IntoIterator$GT$9into_iter17h61da204698638269E"(ptr sret(%"core::ops::range::RangeInclusive<i32>") %_2, ptr %_3)
  call void @llvm.memcpy.p0.p0.i64(ptr align 4 %iter, ptr align 4 %_2, i64 12, i1 false)
  br label %bb3

bb3:                                              ; preds = %bb8, %bb5, %start
  %count.0 = phi i32 [ 0, %start ], [ %5, %bb8 ], [ %count.0, %bb5 ]
  %0 = call { i32, i32 } @"_ZN4core4iter5range110_$LT$impl$u20$core..iter..traits..iterator..Iterator$u20$for$u20$core..ops..range..RangeInclusive$LT$A$GT$$GT$4next17h9ea9c0cd3a033b22E"(ptr align 4 %iter)
  store { i32, i32 } %0, ptr %_5, align 4
  %1 = load i32, ptr %_5, align 4, !range !4, !noundef !3
  %_7 = zext i32 %1 to i64
  %2 = icmp eq i64 %_7, 0
  br i1 %2, label %bb7, label %bb5

bb7:                                              ; preds = %bb3
  ret i32 %count.0

bb5:                                              ; preds = %bb3
  %3 = getelementptr inbounds { i32, i32 }, ptr %_5, i32 0, i32 1
  %i = load i32, ptr %3, align 4, !noundef !3
  %_9 = srem i32 %i, 2
  %4 = icmp eq i32 %_9, 0
  br i1 %4, label %bb8, label %bb3

bb6:                                              ; No predecessors!
  unreachable

bb8:                                              ; preds = %bb5
  %5 = add i32 %count.0, 1
  br label %bb3
}

; Function Attrs: nonlazybind uwtable
define i32 @klee_harness() unnamed_addr #1 {
start:
  %n = alloca i32, align 4
  store i32 0, ptr %n, align 4
  call void @klee_make_symbolic(ptr %n, i64 4, ptr @alloc_e01bdcd616f29df38e098e75c85b494d)
  %_14 = load i32, ptr %n, align 4, !noundef !3
  %_13 = icmp sge i32 %_14, 0
  br i1 %_13, label %bb5, label %bb4

bb4:                                              ; preds = %start
  br label %bb6

bb5:                                              ; preds = %start
  %_16 = load i32, ptr %n, align 4, !noundef !3
  %_15 = icmp sle i32 %_16, 100
  %0 = zext i1 %_15 to i8
  br label %bb6

bb6:                                              ; preds = %bb5, %bb4
  %_12.0 = phi i8 [ %0, %bb5 ], [ 0, %bb4 ]
  %1 = trunc i8 %_12.0 to i1
  %_11 = zext i1 %1 to i32
  call void @klee_assume(i32 %_11)
  %_17 = load i32, ptr %n, align 4, !noundef !3
  %2 = call i32 @_ZN23count_even_rust_harness10count_even17h9dfdce1085948c80E(i32 %_17)
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
!2 = !{i8 0, i8 2}
!3 = !{}
!4 = !{i32 0, i32 2}

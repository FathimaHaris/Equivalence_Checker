; ModuleID = '/tmp/equivalence_checker/compute_c_opt_display.bc'
source_filename = "/tmp/equivalence_checker/compute_c_harness.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

@.str = private unnamed_addr constant [2 x i8] c"x\00", align 1
@.str.1 = private unnamed_addr constant [2 x i8] c"y\00", align 1

; Function Attrs: noinline nounwind uwtable
define dso_local i32 @compute(i32 noundef %0, i32 noundef %1) #0 {
  %3 = icmp sgt i32 %0, 10
  br i1 %3, label %4, label %6

4:                                                ; preds = %2
  %5 = add nsw i32 %0, %1
  br label %8

6:                                                ; preds = %2
  %7 = mul nsw i32 %0, %1
  br label %8

8:                                                ; preds = %6, %4
  %.0 = phi i32 [ %5, %4 ], [ %7, %6 ]
  ret i32 %.0
}

; Function Attrs: noinline nounwind uwtable
define dso_local i32 @main() #0 {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  call void @klee_make_symbolic(ptr noundef %1, i64 noundef 4, ptr noundef @.str)
  call void @klee_make_symbolic(ptr noundef %2, i64 noundef 4, ptr noundef @.str.1)
  %4 = load i32, ptr %1, align 4
  %5 = icmp sge i32 %4, 0
  br i1 %5, label %6, label %9

6:                                                ; preds = %0
  %7 = load i32, ptr %1, align 4
  %8 = icmp sle i32 %7, 100
  br label %9

9:                                                ; preds = %6, %0
  %10 = phi i1 [ false, %0 ], [ %8, %6 ]
  %11 = zext i1 %10 to i32
  %12 = sext i32 %11 to i64
  call void @klee_assume(i64 noundef %12)
  %13 = load i32, ptr %2, align 4
  %14 = icmp sge i32 %13, 0
  br i1 %14, label %15, label %18

15:                                               ; preds = %9
  %16 = load i32, ptr %2, align 4
  %17 = icmp sle i32 %16, 100
  br label %18

18:                                               ; preds = %15, %9
  %19 = phi i1 [ false, %9 ], [ %17, %15 ]
  %20 = zext i1 %19 to i32
  %21 = sext i32 %20 to i64
  call void @klee_assume(i64 noundef %21)
  %22 = load i32, ptr %1, align 4
  %23 = load i32, ptr %2, align 4
  %24 = call i32 @compute(i32 noundef %22, i32 noundef %23)
  store volatile i32 %24, ptr %3, align 4
  %25 = load volatile i32, ptr %3, align 4
  ret i32 %25
}

declare void @klee_make_symbolic(ptr noundef, i64 noundef, ptr noundef) #1

declare void @klee_assume(i64 noundef) #1

attributes #0 = { noinline nounwind uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }
attributes #1 = { "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }

!llvm.module.flags = !{!0, !1, !2, !3, !4}
!llvm.ident = !{!5}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{i32 7, !"PIC Level", i32 2}
!2 = !{i32 7, !"PIE Level", i32 2}
!3 = !{i32 7, !"uwtable", i32 2}
!4 = !{i32 7, !"frame-pointer", i32 2}
!5 = !{!"Ubuntu clang version 15.0.7"}

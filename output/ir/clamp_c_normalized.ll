; ModuleID = '/tmp/equivalence_checker/clamp_c_opt_display.bc'
source_filename = "/tmp/equivalence_checker/clamp_c_harness.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

@.str = private unnamed_addr constant [2 x i8] c"x\00", align 1
@.str.1 = private unnamed_addr constant [3 x i8] c"lo\00", align 1
@.str.2 = private unnamed_addr constant [3 x i8] c"hi\00", align 1

; Function Attrs: noinline nounwind uwtable
define dso_local i32 @clamp(i32 noundef %0, i32 noundef %1, i32 noundef %2) #0 {
  %4 = icmp slt i32 %0, %1
  br i1 %4, label %5, label %6

5:                                                ; preds = %3
  br label %10

6:                                                ; preds = %3
  %7 = icmp sgt i32 %0, %2
  br i1 %7, label %8, label %9

8:                                                ; preds = %6
  br label %10

9:                                                ; preds = %6
  br label %10

10:                                               ; preds = %9, %8, %5
  %.0 = phi i32 [ %1, %5 ], [ %2, %8 ], [ %0, %9 ]
  ret i32 %.0
}

; Function Attrs: noinline nounwind uwtable
define dso_local i32 @main() #0 {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  call void @klee_make_symbolic(ptr noundef %1, i64 noundef 4, ptr noundef @.str)
  call void @klee_make_symbolic(ptr noundef %2, i64 noundef 4, ptr noundef @.str.1)
  call void @klee_make_symbolic(ptr noundef %3, i64 noundef 4, ptr noundef @.str.2)
  %5 = load i32, ptr %1, align 4
  %6 = icmp sge i32 %5, 0
  br i1 %6, label %7, label %10

7:                                                ; preds = %0
  %8 = load i32, ptr %1, align 4
  %9 = icmp sle i32 %8, 100
  br label %10

10:                                               ; preds = %7, %0
  %11 = phi i1 [ false, %0 ], [ %9, %7 ]
  %12 = zext i1 %11 to i32
  %13 = sext i32 %12 to i64
  call void @klee_assume(i64 noundef %13)
  %14 = load i32, ptr %2, align 4
  %15 = icmp sge i32 %14, 0
  br i1 %15, label %16, label %19

16:                                               ; preds = %10
  %17 = load i32, ptr %2, align 4
  %18 = icmp sle i32 %17, 100
  br label %19

19:                                               ; preds = %16, %10
  %20 = phi i1 [ false, %10 ], [ %18, %16 ]
  %21 = zext i1 %20 to i32
  %22 = sext i32 %21 to i64
  call void @klee_assume(i64 noundef %22)
  %23 = load i32, ptr %3, align 4
  %24 = icmp sge i32 %23, 0
  br i1 %24, label %25, label %28

25:                                               ; preds = %19
  %26 = load i32, ptr %3, align 4
  %27 = icmp sle i32 %26, 100
  br label %28

28:                                               ; preds = %25, %19
  %29 = phi i1 [ false, %19 ], [ %27, %25 ]
  %30 = zext i1 %29 to i32
  %31 = sext i32 %30 to i64
  call void @klee_assume(i64 noundef %31)
  %32 = load i32, ptr %1, align 4
  %33 = load i32, ptr %2, align 4
  %34 = load i32, ptr %3, align 4
  %35 = call i32 @clamp(i32 noundef %32, i32 noundef %33, i32 noundef %34)
  store volatile i32 %35, ptr %4, align 4
  %36 = load volatile i32, ptr %4, align 4
  ret i32 %36
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

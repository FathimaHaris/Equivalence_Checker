; ModuleID = '/tmp/equivalence_checker/categorize_c_harness.c'
source_filename = "/tmp/equivalence_checker/categorize_c_harness.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

@.str = private unnamed_addr constant [2 x i8] c"a\00", align 1
@.str.1 = private unnamed_addr constant [2 x i8] c"b\00", align 1
@.str.2 = private unnamed_addr constant [7 x i8] c"result\00", align 1

; Function Attrs: noinline nounwind uwtable
define dso_local i32 @categorize(i32 noundef %0, i32 noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store i32 %0, ptr %4, align 4
  store i32 %1, ptr %5, align 4
  %6 = load i32, ptr %4, align 4
  %7 = load i32, ptr %5, align 4
  %8 = icmp sgt i32 %6, %7
  br i1 %8, label %9, label %14

9:                                                ; preds = %2
  %10 = load i32, ptr %4, align 4
  %11 = icmp sgt i32 %10, 10
  br i1 %11, label %12, label %13

12:                                               ; preds = %9
  store i32 3, ptr %3, align 4
  br label %19

13:                                               ; preds = %9
  store i32 2, ptr %3, align 4
  br label %19

14:                                               ; preds = %2
  %15 = load i32, ptr %5, align 4
  %16 = icmp sgt i32 %15, 10
  br i1 %16, label %17, label %18

17:                                               ; preds = %14
  store i32 -1, ptr %3, align 4
  br label %19

18:                                               ; preds = %14
  store i32 0, ptr %3, align 4
  br label %19

19:                                               ; preds = %18, %17, %13, %12
  %20 = load i32, ptr %3, align 4
  ret i32 %20
}

; Function Attrs: noinline nounwind uwtable
define dso_local i32 @main() #0 {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  %4 = alloca [1 x i32], align 4
  store i32 0, ptr %1, align 4
  call void @klee_make_symbolic(ptr noundef %2, i64 noundef 4, ptr noundef @.str)
  call void @klee_make_symbolic(ptr noundef %3, i64 noundef 4, ptr noundef @.str.1)
  %5 = load i32, ptr %2, align 4
  %6 = icmp sge i32 %5, 0
  br i1 %6, label %7, label %10

7:                                                ; preds = %0
  %8 = load i32, ptr %2, align 4
  %9 = icmp sle i32 %8, 100
  br label %10

10:                                               ; preds = %7, %0
  %11 = phi i1 [ false, %0 ], [ %9, %7 ]
  %12 = zext i1 %11 to i32
  %13 = sext i32 %12 to i64
  call void @klee_assume(i64 noundef %13)
  %14 = load i32, ptr %3, align 4
  %15 = icmp sge i32 %14, 0
  br i1 %15, label %16, label %19

16:                                               ; preds = %10
  %17 = load i32, ptr %3, align 4
  %18 = icmp sle i32 %17, 100
  br label %19

19:                                               ; preds = %16, %10
  %20 = phi i1 [ false, %10 ], [ %18, %16 ]
  %21 = zext i1 %20 to i32
  %22 = sext i32 %21 to i64
  call void @klee_assume(i64 noundef %22)
  %23 = getelementptr inbounds [1 x i32], ptr %4, i64 0, i64 0
  call void @klee_make_symbolic(ptr noundef %23, i64 noundef 4, ptr noundef @.str.2)
  %24 = getelementptr inbounds [1 x i32], ptr %4, i64 0, i64 0
  %25 = load i32, ptr %24, align 4
  %26 = load i32, ptr %2, align 4
  %27 = load i32, ptr %3, align 4
  %28 = call i32 @categorize(i32 noundef %26, i32 noundef %27)
  %29 = icmp eq i32 %25, %28
  %30 = zext i1 %29 to i32
  %31 = sext i32 %30 to i64
  call void @klee_assume(i64 noundef %31)
  %32 = getelementptr inbounds [1 x i32], ptr %4, i64 0, i64 0
  %33 = load i32, ptr %32, align 4
  ret i32 %33
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

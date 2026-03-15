; ModuleID = '/tmp/equivalence_checker/in_range_c_harness.c'
source_filename = "/tmp/equivalence_checker/in_range_c_harness.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

@.str = private unnamed_addr constant [2 x i8] c"x\00", align 1
@.str.1 = private unnamed_addr constant [3 x i8] c"lo\00", align 1
@.str.2 = private unnamed_addr constant [3 x i8] c"hi\00", align 1
@.str.3 = private unnamed_addr constant [7 x i8] c"result\00", align 1

; Function Attrs: noinline nounwind uwtable
define dso_local i32 @in_range(i32 noundef %0, i32 noundef %1, i32 noundef %2) #0 {
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  store i32 %0, ptr %4, align 4
  store i32 %1, ptr %5, align 4
  store i32 %2, ptr %6, align 4
  %7 = load i32, ptr %4, align 4
  %8 = load i32, ptr %5, align 4
  %9 = icmp sge i32 %7, %8
  br i1 %9, label %10, label %14

10:                                               ; preds = %3
  %11 = load i32, ptr %4, align 4
  %12 = load i32, ptr %6, align 4
  %13 = icmp sle i32 %11, %12
  br label %14

14:                                               ; preds = %10, %3
  %15 = phi i1 [ false, %3 ], [ %13, %10 ]
  %16 = zext i1 %15 to i64
  %17 = select i1 %15, i32 1, i32 0
  ret i32 %17
}

; Function Attrs: noinline nounwind uwtable
define dso_local i32 @main() #0 {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca [1 x i32], align 4
  store i32 0, ptr %1, align 4
  call void @klee_make_symbolic(ptr noundef %2, i64 noundef 4, ptr noundef @.str)
  call void @klee_make_symbolic(ptr noundef %3, i64 noundef 4, ptr noundef @.str.1)
  call void @klee_make_symbolic(ptr noundef %4, i64 noundef 4, ptr noundef @.str.2)
  %6 = load i32, ptr %2, align 4
  %7 = icmp sge i32 %6, 0
  br i1 %7, label %8, label %11

8:                                                ; preds = %0
  %9 = load i32, ptr %2, align 4
  %10 = icmp sle i32 %9, 100
  br label %11

11:                                               ; preds = %8, %0
  %12 = phi i1 [ false, %0 ], [ %10, %8 ]
  %13 = zext i1 %12 to i32
  %14 = sext i32 %13 to i64
  call void @klee_assume(i64 noundef %14)
  %15 = load i32, ptr %3, align 4
  %16 = icmp sge i32 %15, 0
  br i1 %16, label %17, label %20

17:                                               ; preds = %11
  %18 = load i32, ptr %3, align 4
  %19 = icmp sle i32 %18, 100
  br label %20

20:                                               ; preds = %17, %11
  %21 = phi i1 [ false, %11 ], [ %19, %17 ]
  %22 = zext i1 %21 to i32
  %23 = sext i32 %22 to i64
  call void @klee_assume(i64 noundef %23)
  %24 = load i32, ptr %4, align 4
  %25 = icmp sge i32 %24, 0
  br i1 %25, label %26, label %29

26:                                               ; preds = %20
  %27 = load i32, ptr %4, align 4
  %28 = icmp sle i32 %27, 100
  br label %29

29:                                               ; preds = %26, %20
  %30 = phi i1 [ false, %20 ], [ %28, %26 ]
  %31 = zext i1 %30 to i32
  %32 = sext i32 %31 to i64
  call void @klee_assume(i64 noundef %32)
  %33 = getelementptr inbounds [1 x i32], ptr %5, i64 0, i64 0
  call void @klee_make_symbolic(ptr noundef %33, i64 noundef 4, ptr noundef @.str.3)
  %34 = getelementptr inbounds [1 x i32], ptr %5, i64 0, i64 0
  %35 = load i32, ptr %34, align 4
  %36 = load i32, ptr %2, align 4
  %37 = load i32, ptr %3, align 4
  %38 = load i32, ptr %4, align 4
  %39 = call i32 @in_range(i32 noundef %36, i32 noundef %37, i32 noundef %38)
  %40 = icmp eq i32 %35, %39
  %41 = zext i1 %40 to i32
  %42 = sext i32 %41 to i64
  call void @klee_assume(i64 noundef %42)
  %43 = getelementptr inbounds [1 x i32], ptr %5, i64 0, i64 0
  %44 = load i32, ptr %43, align 4
  ret i32 %44
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

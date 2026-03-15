; ModuleID = '/tmp/equivalence_checker/linear_search_c_harness.c'
source_filename = "/tmp/equivalence_checker/linear_search_c_harness.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

@.str = private unnamed_addr constant [7 x i8] c"target\00", align 1
@.str.1 = private unnamed_addr constant [7 x i8] c"result\00", align 1

; Function Attrs: noinline nounwind uwtable
define dso_local i32 @linear_search(i32 noundef %0) #0 {
  %2 = alloca i32, align 4
  %3 = alloca [6 x i32], align 16
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store i32 %0, ptr %2, align 4
  %6 = getelementptr inbounds [6 x i32], ptr %3, i64 0, i64 0
  store i32 3, ptr %6, align 16
  %7 = getelementptr inbounds [6 x i32], ptr %3, i64 0, i64 1
  store i32 7, ptr %7, align 4
  %8 = getelementptr inbounds [6 x i32], ptr %3, i64 0, i64 2
  store i32 1, ptr %8, align 8
  %9 = getelementptr inbounds [6 x i32], ptr %3, i64 0, i64 3
  store i32 9, ptr %9, align 4
  %10 = getelementptr inbounds [6 x i32], ptr %3, i64 0, i64 4
  store i32 4, ptr %10, align 16
  %11 = getelementptr inbounds [6 x i32], ptr %3, i64 0, i64 5
  store i32 6, ptr %11, align 4
  store i32 0, ptr %4, align 4
  store i32 -1, ptr %5, align 4
  br label %12

12:                                               ; preds = %24, %1
  %13 = load i32, ptr %4, align 4
  %14 = icmp slt i32 %13, 6
  br i1 %14, label %15, label %27

15:                                               ; preds = %12
  %16 = load i32, ptr %4, align 4
  %17 = sext i32 %16 to i64
  %18 = getelementptr inbounds [6 x i32], ptr %3, i64 0, i64 %17
  %19 = load i32, ptr %18, align 4
  %20 = load i32, ptr %2, align 4
  %21 = icmp eq i32 %19, %20
  br i1 %21, label %22, label %24

22:                                               ; preds = %15
  %23 = load i32, ptr %4, align 4
  store i32 %23, ptr %5, align 4
  br label %27

24:                                               ; preds = %15
  %25 = load i32, ptr %4, align 4
  %26 = add nsw i32 %25, 1
  store i32 %26, ptr %4, align 4
  br label %12, !llvm.loop !6

27:                                               ; preds = %22, %12
  %28 = load i32, ptr %5, align 4
  ret i32 %28
}

; Function Attrs: noinline nounwind uwtable
define dso_local i32 @main() #0 {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  %3 = alloca [1 x i32], align 4
  store i32 0, ptr %1, align 4
  call void @klee_make_symbolic(ptr noundef %2, i64 noundef 4, ptr noundef @.str)
  %4 = load i32, ptr %2, align 4
  %5 = icmp sge i32 %4, 0
  br i1 %5, label %6, label %9

6:                                                ; preds = %0
  %7 = load i32, ptr %2, align 4
  %8 = icmp sle i32 %7, 100
  br label %9

9:                                                ; preds = %6, %0
  %10 = phi i1 [ false, %0 ], [ %8, %6 ]
  %11 = zext i1 %10 to i32
  %12 = sext i32 %11 to i64
  call void @klee_assume(i64 noundef %12)
  %13 = getelementptr inbounds [1 x i32], ptr %3, i64 0, i64 0
  call void @klee_make_symbolic(ptr noundef %13, i64 noundef 4, ptr noundef @.str.1)
  %14 = getelementptr inbounds [1 x i32], ptr %3, i64 0, i64 0
  %15 = load i32, ptr %14, align 4
  %16 = load i32, ptr %2, align 4
  %17 = call i32 @linear_search(i32 noundef %16)
  %18 = icmp eq i32 %15, %17
  %19 = zext i1 %18 to i32
  %20 = sext i32 %19 to i64
  call void @klee_assume(i64 noundef %20)
  %21 = getelementptr inbounds [1 x i32], ptr %3, i64 0, i64 0
  %22 = load i32, ptr %21, align 4
  ret i32 %22
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
!6 = distinct !{!6, !7}
!7 = !{!"llvm.loop.mustprogress"}

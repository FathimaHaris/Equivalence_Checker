; ModuleID = '/tmp/equivalence_checker/linear_search_c_opt_display.bc'
source_filename = "/tmp/equivalence_checker/linear_search_c_harness.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

@.str = private unnamed_addr constant [7 x i8] c"target\00", align 1
@.str.1 = private unnamed_addr constant [7 x i8] c"result\00", align 1

; Function Attrs: noinline nounwind uwtable
define dso_local i32 @linear_search(i32 noundef %0) #0 {
  %2 = alloca [6 x i32], align 16
  %3 = getelementptr inbounds [6 x i32], ptr %2, i64 0, i64 0
  store i32 3, ptr %3, align 16
  %4 = getelementptr inbounds [6 x i32], ptr %2, i64 0, i64 1
  store i32 7, ptr %4, align 4
  %5 = getelementptr inbounds [6 x i32], ptr %2, i64 0, i64 2
  store i32 1, ptr %5, align 8
  %6 = getelementptr inbounds [6 x i32], ptr %2, i64 0, i64 3
  store i32 9, ptr %6, align 4
  %7 = getelementptr inbounds [6 x i32], ptr %2, i64 0, i64 4
  store i32 4, ptr %7, align 16
  %8 = getelementptr inbounds [6 x i32], ptr %2, i64 0, i64 5
  store i32 6, ptr %8, align 4
  br label %9

9:                                                ; preds = %17, %1
  %.01 = phi i32 [ 0, %1 ], [ %18, %17 ]
  %10 = icmp slt i32 %.01, 6
  br i1 %10, label %11, label %19

11:                                               ; preds = %9
  %12 = sext i32 %.01 to i64
  %13 = getelementptr inbounds [6 x i32], ptr %2, i64 0, i64 %12
  %14 = load i32, ptr %13, align 4
  %15 = icmp eq i32 %14, %0
  br i1 %15, label %16, label %17

16:                                               ; preds = %11
  br label %19

17:                                               ; preds = %11
  %18 = add nsw i32 %.01, 1
  br label %9, !llvm.loop !6

19:                                               ; preds = %16, %9
  %.0 = phi i32 [ %.01, %16 ], [ -1, %9 ]
  ret i32 %.0
}

; Function Attrs: noinline nounwind uwtable
define dso_local i32 @main() #0 {
  %1 = alloca i32, align 4
  %2 = alloca [1 x i32], align 4
  call void @klee_make_symbolic(ptr noundef %1, i64 noundef 4, ptr noundef @.str)
  %3 = load i32, ptr %1, align 4
  %4 = icmp sge i32 %3, 0
  br i1 %4, label %5, label %8

5:                                                ; preds = %0
  %6 = load i32, ptr %1, align 4
  %7 = icmp sle i32 %6, 100
  br label %8

8:                                                ; preds = %5, %0
  %9 = phi i1 [ false, %0 ], [ %7, %5 ]
  %10 = zext i1 %9 to i32
  %11 = sext i32 %10 to i64
  call void @klee_assume(i64 noundef %11)
  %12 = getelementptr inbounds [1 x i32], ptr %2, i64 0, i64 0
  call void @klee_make_symbolic(ptr noundef %12, i64 noundef 4, ptr noundef @.str.1)
  %13 = getelementptr inbounds [1 x i32], ptr %2, i64 0, i64 0
  %14 = load i32, ptr %13, align 4
  %15 = load i32, ptr %1, align 4
  %16 = call i32 @linear_search(i32 noundef %15)
  %17 = icmp eq i32 %14, %16
  %18 = zext i1 %17 to i32
  %19 = sext i32 %18 to i64
  call void @klee_assume(i64 noundef %19)
  %20 = getelementptr inbounds [1 x i32], ptr %2, i64 0, i64 0
  %21 = load i32, ptr %20, align 4
  ret i32 %21
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

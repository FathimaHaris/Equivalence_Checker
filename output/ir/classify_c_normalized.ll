; ModuleID = '/tmp/equivalence_checker/classify_c_normalized.bc'
source_filename = "/tmp/equivalence_checker/classify_c_harness.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

@.str = private unnamed_addr constant [2 x i8] c"x\00", align 1

; Function Attrs: noinline nounwind uwtable
define dso_local i32 @classify(i32 noundef %0) #0 {
  %2 = icmp slt i32 %0, 0
  br i1 %2, label %3, label %4

3:                                                ; preds = %1
  br label %14

4:                                                ; preds = %1
  %5 = icmp eq i32 %0, 0
  br i1 %5, label %6, label %7

6:                                                ; preds = %4
  br label %14

7:                                                ; preds = %4
  %8 = icmp slt i32 %0, 10
  br i1 %8, label %9, label %10

9:                                                ; preds = %7
  br label %14

10:                                               ; preds = %7
  %11 = icmp slt i32 %0, 100
  br i1 %11, label %12, label %13

12:                                               ; preds = %10
  br label %14

13:                                               ; preds = %10
  br label %14

14:                                               ; preds = %13, %12, %9, %6, %3
  %.0 = phi i32 [ -1, %3 ], [ 0, %6 ], [ 1, %9 ], [ 2, %12 ], [ 3, %13 ]
  ret i32 %.0
}

; Function Attrs: noinline nounwind uwtable
define dso_local i32 @main() #0 {
  %1 = alloca i32, align 4
  call void @klee_make_symbolic(ptr noundef %1, i64 noundef 4, ptr noundef @.str)
  %2 = load i32, ptr %1, align 4
  %3 = icmp sge i32 %2, -10
  br i1 %3, label %4, label %7

4:                                                ; preds = %0
  %5 = load i32, ptr %1, align 4
  %6 = icmp sle i32 %5, 110
  br label %7

7:                                                ; preds = %4, %0
  %8 = phi i1 [ false, %0 ], [ %6, %4 ]
  %9 = zext i1 %8 to i32
  %10 = sext i32 %9 to i64
  call void @klee_assume(i64 noundef %10)
  %11 = load i32, ptr %1, align 4
  %12 = call i32 @classify(i32 noundef %11)
  ret i32 %12
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

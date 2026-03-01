; ModuleID = '/tmp/equivalence_checker/max_c_normalized.bc'
source_filename = "/tmp/equivalence_checker/max_c_harness.c"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

@.str = private unnamed_addr constant [2 x i8] c"x\00", align 1
@.str.1 = private unnamed_addr constant [2 x i8] c"y\00", align 1

; Function Attrs: noinline nounwind uwtable
define dso_local i32 @max(i32 noundef %0, i32 noundef %1) #0 {
  %3 = icmp sgt i32 %0, %1
  br i1 %3, label %4, label %5

4:                                                ; preds = %2
  br label %6

5:                                                ; preds = %2
  br label %6

6:                                                ; preds = %5, %4
  %.0 = phi i32 [ %0, %4 ], [ %1, %5 ]
  ret i32 %.0
}

; Function Attrs: noinline nounwind uwtable
define dso_local i32 @main() #0 {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  call void @klee_make_symbolic(ptr noundef %1, i64 noundef 4, ptr noundef @.str)
  call void @klee_make_symbolic(ptr noundef %2, i64 noundef 4, ptr noundef @.str.1)
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
  %12 = load i32, ptr %2, align 4
  %13 = icmp sge i32 %12, 0
  br i1 %13, label %14, label %17

14:                                               ; preds = %8
  %15 = load i32, ptr %2, align 4
  %16 = icmp sle i32 %15, 100
  br label %17

17:                                               ; preds = %14, %8
  %18 = phi i1 [ false, %8 ], [ %16, %14 ]
  %19 = zext i1 %18 to i32
  %20 = sext i32 %19 to i64
  call void @klee_assume(i64 noundef %20)
  %21 = load i32, ptr %1, align 4
  %22 = load i32, ptr %2, align 4
  %23 = call i32 @max(i32 noundef %21, i32 noundef %22)
  ret i32 %23
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

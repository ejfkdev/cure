package com.example.expression;

public sealed interface Expr permits ConstantExpr, PlusExpr, TimesExpr, NegExpr {
}

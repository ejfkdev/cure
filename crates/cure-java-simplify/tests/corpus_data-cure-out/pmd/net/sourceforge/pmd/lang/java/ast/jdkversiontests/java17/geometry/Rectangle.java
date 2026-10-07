package com.example.geometry;

public sealed class Rectangle extends Shape permits TransparentRectangle, FilledRectangle {
}

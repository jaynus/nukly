#ifndef NUKLY_MATH_H
#define NUKLY_MATH_H

#define FORCE_EVAL(x) do {                        \
	if (sizeof(x) == sizeof(float)) {         \
		volatile float __x;               \
		__x = (x);                        \
                (void)__x;                        \
	} else if (sizeof(x) == sizeof(double)) { \
		volatile double __x;              \
		__x = (x);                        \
                (void)__x;                        \
	} else {                                  \
		volatile long double __x;         \
		__x = (x);                        \
                (void)__x;                        \
	}                                         \
} while(0)

static double floor(double x)
{
    union {double f; unsigned long long i;} u = {x};
    int e = u.i >> 52 & 0x7ff;
    double y;

    if (e >= 0x3ff+52 || x == 0)
        return x;
    /* y = int(x) - x, where int(x) is an integer neighbor of x */
    if (u.i >> 63)
        y = (double)(x - 0x1p52) + 0x1p52 - x;
    else
        y = (double)(x + 0x1p52) - 0x1p52 - x;
    /* special case because of non-nearest rounding modes */
    if (e <= 0x3ff-1) {
        FORCE_EVAL(y);
        return u.i >> 63 ? -1 : 0;
    }
    if (y > 0)
        return x + y - 1;
    return x + y;
}

#endif //NUKLY_MATH_H
#define NULL 0

#include "src/math.h"

#define NK_ASSERT(expr) if(!(expr)) { int * ptr = 0; int abc = *ptr; };
#define STBRP_ASSERT(expr)
#define STBTT_assert(expr)

#define STBRP_SORT qsort

#define STBTT_ifloor(x)   ((int) floor(x))
#define STBTT_iceil(x)    ((int) ceil(x))
#define STBTT_sqrt(x)      sqrt(x)
#define STBTT_pow(x,y)     pow(x,y)
#define STBTT_fmod(x,y)    fmod(x,y)
#define STBTT_cos(x)       cos(x)
#define STBTT_acos(x)      acos(x)
#define STBTT_fabs(x)      fabs(x)
#define STBTT_strlen(x)    strlen(x)
#define STBTT_memcpy       memcpy
#define STBTT_memset       memset


typedef unsigned long long size_t;
typedef int (*cmpfun)(const void *, const void *);

extern void *__nukly_alloc_proxy(void * const handle, void *ptr, int size);
extern void *__nukly_free_proxy(void * const handle, void *ptr);

#define STBTT_malloc(x, u) __nukly_alloc_proxy(u, 0, x)
#define STBTT_free(x, u) __nukly_free_proxy(u, x)

extern void qsort(void *base, size_t nel, size_t width, cmpfun cmp);

#include <nuklear.h>



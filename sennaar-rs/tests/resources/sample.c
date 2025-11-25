#define MAKRO (114514)
#define BIG_MACRO 1145141919810ULL
#define BAD(s) s + s

enum WHAT {
  FOO, BAR, BAZ = 114514
};

struct Foo {
  int *aa;
};

typedef unsigned long long ull;

typedef struct _what * what_handle;
typedef struct how * how;

typedef void (*MAIN)(int, char**);

typedef struct Wrap {
  int value;
} Wrap;


typedef struct {
  int walue;
} Wrap2, *Wrap3, Wrap2d5;

typedef struct {
  int ualue;
} Wrap4;

struct {
  int yalue;
};

typedef struct {
  int xalue;
} *Wrap5;

// struct {
//   int zalue;
//   struct {
//     int aalue;
//   } aalue_s;

//   union {
//     int balue;
//     int calue;
//   };
// };


void foo(int a, int b) {
  int arr[5];
  // comment to make CI happy, uncomment to test sizeof
  // int int_size = sizeof(int);
  arr[0] = a++;
  arr[0x01] = ++b;
  arr[2] = 1 ? !MAKRO : 0;
  foo(&arr, *arr);
  (int) 0x114514;
  a += b;
}

int callback(int (*f)(unsigned int, int), int arr[]) {
  return 0;
}

void char_hell(unsigned char uchar, char char_s, signed char schar) {
}

void elaborate(ull b, struct Foo* foo, enum WHAT what) {
}

void konst_hell(
  const char c, 
  const int arr[5],
  int arrr[const],
  int * const p,
  const int * const pp,
  const int * const * const * const ppp
) {
}

int noproto() {
  return 0;
}
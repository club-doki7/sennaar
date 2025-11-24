#define MAKRO (114514)
#define BIG_MACRO 1145141919810ULL

enum WHAT {
  FOO, BAR
};

struct Foo {
  int *aa;
};

typedef unsigned long long ull;

void foo(int a, int b) {
  int arr[5];
  int int_size = sizeof(int);
  arr[0] = a++;
  arr[0x01] = ++b;
  arr[2] = 1 ? !MAKRO : 0;
  foo(&arr, *arr);
  (int) 0x114514;
  a += b;
}

int callback(int (*f)(unsigned int, int), int arr[10]) {
  return 0;
}

void char_hell(unsigned char uchar, char char_s, signed char schar) {
}

void elaborate(ull b, struct Foo* foo, enum WHAT what) {
}

int noproto() {
  return 0;
}
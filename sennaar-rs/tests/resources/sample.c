#define MAKRO (114514)
#define BIG_MACRO 1145141919810ULL

typedef unsigned long long ull;

int foo(int a, int b) {
  char c = '\n';
  int arr[5];
  arr[0] = a++;
  arr[0x01] = ++b;
  arr[2] = 1 ? !MAKRO : 0;
  ull what = BIG_MACRO;
  return arr[0] + arr[1];
}
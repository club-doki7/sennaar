struct Named {
    int value;
};

typedef struct { int value } TypedefUnnamed;
typedef struct TypedefNamed { int value } TypedefNamed;
typedef void (*FunctionPointer)(int i);
typedef void Function(char j);
typedef struct _what whatOpaqueTypedef;
typedef struct _how * howOpaqueHanle;

struct Nest {
    int a;
    char b;
};

void someCommand(int a, const char *b);
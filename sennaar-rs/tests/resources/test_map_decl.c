// uncomment thses until we can handle them

struct {
    int value;
} topVarDecl;

struct Named {
    int value;
};

typedef struct { int value } TypedefUnnamed;
typedef struct TypedefNamed { int value } TypedefNamed;

struct Nest {
    struct { int value } walue, *pvalue;
    void (*f)(struct { int value } palue);
    int ualue;
    union {
        int indirect0;
        int indirect1;
    };
}
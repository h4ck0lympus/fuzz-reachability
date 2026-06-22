#include <stddef.h>
#include <stdint.h>

/* Mirrors libtiff's codec dispatch closely enough to trip SVF's points-to:
 * a struct with many same-typed function-pointer fields whose defaults are set
 * by one function and overridden by codec init functions reached through a
 * global table (plus a dynamic, registered linked list). The override stores a
 * function address into a field that is later called indirectly. SVF loses the
 * store->load connection across these layers and drops the real target; the
 * memory-escape set recovers it so the SVF backend stays sound. */

typedef struct Obj Obj;
typedef int (*method)(Obj *);

struct Obj {
    method a, b, c, d, e, f, g, h; /* many same-typed slots, like TIFF */
    int v;
};

typedef int (*initfn)(Obj *, int);

typedef struct Codec Codec;
struct Codec {
    int key;
    initfn init;
    Codec *next;
};

static int def_a(Obj *o) { return 0; }
static int def_b(Obj *o) { return 0; }
static int def_c(Obj *o) { return 0; }
static int def_d(Obj *o) { return 0; }
static int def_e(Obj *o) { return 0; }
static int def_f(Obj *o) { return 0; }
static int def_g(Obj *o) { return 0; }
static int def_h(Obj *o) { return 0; }

static int real_run(Obj *o) { return o->v + 1; } /* reached only via o->e */

static void set_defaults(Obj *o) {
    o->a = def_a; o->b = def_b; o->c = def_c; o->d = def_d;
    o->e = def_e; o->f = def_f; o->g = def_g; o->h = def_h;
}

static int init_a(Obj *o, int x) {
    o->e = real_run; /* override one field with the real target */
    o->v = x;
    return 0;
}

static Codec *registered; /* populated dynamically, like registeredCODECS */
static const Codec builtin[] = {{1, init_a, 0}, {0, 0, 0}};

static const Codec *find(int key) {
    for (Codec *r = registered; r; r = r->next)
        if (r->key == key)
            return r;
    for (const Codec *c = builtin; c->init; ++c)
        if (c->key == key)
            return c;
    return 0;
}

int dead_run(Obj *o) { return o->v - 1; } /* never address-taken, never called */

int LLVMFuzzerTestOneInput(const uint8_t *d, size_t n) {
    Obj o;
    set_defaults(&o);
    const Codec *c = find(n ? d[0] : 1);
    if (c)
        c->init(&o, (int)n); /* indirect #1 -> init_a (via the codec table) */
    return o.e ? o.e(&o) : 0; /* indirect #2 -> real_run (via field override) */
}

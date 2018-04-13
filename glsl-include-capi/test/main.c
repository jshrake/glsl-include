#include <stdio.h>
#include "../include/gli.h"

int main() {
  // example 1
  {
    printf("!!!!!\nBasic usage\n!!!!!\n");
    char const* src = "#include <A.glsl>\n#include <B.glsl>\nvoid main() {}";
    struct gli_ctx* ctx = gli_ctx_new();
    gli_include(ctx, "A.glsl", "void A() {}");
    gli_include(ctx, "B.glsl", "void B() {}");
    char const* expanded = gli_expand_to_str(ctx, src);
    if (!expanded) {
      char const* err = gli_get_error_str(ctx);
      printf("Error: %s\n", err);
      gli_str_free(err);
    } else {
      printf("Expanded src:\n%s\n", expanded);
    }
    gli_str_free(expanded);
    gli_ctx_free(ctx);
  }
  // example 2
  {
    printf("!!!!!\nError\n!!!!!\n");
    char const* src = "#include <A.glsl>\n#include <B.glsl>\nvoid main() {}";
    struct gli_ctx* ctx = gli_ctx_new();
    char const* expanded = gli_expand_to_str(ctx, src);
    if (!expanded) {
      char const* err = gli_get_error_str(ctx);
      printf("Error: %s\n", err);
      gli_str_free(err);
    } else {
      printf("Expanded src:\n%s\n", expanded);
    }
    gli_str_free(expanded);
    gli_ctx_free(ctx);
  }
  // example 3
  {
    printf("!!!!!\nSource Maps\n!!!!!\n");
    char const* src = "#include <A.glsl>\nvoid main() {}";
    struct gli_ctx* ctx = gli_ctx_new();
    gli_include(ctx, "A.glsl", "#include <B.glsl>\nvoid A() {}");
    gli_include(ctx, "B.glsl", "#include <C.glsl>\nvoid B() {}");
    gli_include(ctx, "C.glsl", "void C() {}");
    char const* expanded = gli_expand_to_str(ctx, src);
    if (!expanded) {
      char const* err = gli_get_error_str(ctx);
      printf("Error: %s\n", err);
      gli_str_free(err);
    } else {
      printf("Expanded src:\n%s\n", expanded);

      for (int i = 0; i < 4; ++i) {
        char const* origin;
        int line;
        gli_get_source_mapping(ctx, i, &origin, &line);
        printf("Origin of line %d: %s:%d\n", i, origin, line);
        if (origin) gli_str_free(origin);
      }
    }
    gli_str_free(expanded);
    gli_ctx_free(ctx);
  }
  return 0;
}

#ifndef GLSL_INCLUDE_H_
#define GLSL_INCLUDE_H_

struct gli_ctx;
struct gli_ctx *gli_ctx_new();
void gli_ctx_free(struct gli_ctx *ctx);
void gli_str_free(char const *str);

void gli_include(struct gli_ctx *ctx, char const *file, char const *content);
char const *gli_expand_to_str(struct gli_ctx *ctx, char const *src);
void gli_get_source_mapping(struct gli_ctx *ctx, int expanded_line_num,
                            char const **src_file, int *src_file_line_num);

// Call to get the error str if gli_expand_to_str returned null
// Clients must call gli_str_free
char const *gli_get_error_str(struct gli_ctx *ctx);

#endif

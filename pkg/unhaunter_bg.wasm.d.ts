/* tslint:disable */
/* eslint-disable */
export const memory: WebAssembly.Memory;
export function wasm_load(): void;
export function rust_zstd_wasm_shim_qsort(a: number, b: number, c: number, d: number): void;
export function rust_zstd_wasm_shim_malloc(a: number): number;
export function rust_zstd_wasm_shim_memcmp(a: number, b: number, c: number): number;
export function rust_zstd_wasm_shim_calloc(a: number, b: number): number;
export function rust_zstd_wasm_shim_free(a: number): void;
export function rust_zstd_wasm_shim_memcpy(a: number, b: number, c: number): number;
export function rust_zstd_wasm_shim_memmove(a: number, b: number, c: number): number;
export function rust_zstd_wasm_shim_memset(a: number, b: number, c: number): number;
export function wgpu_compute_pass_set_pipeline(a: number, b: number): void;
export function wgpu_compute_pass_set_bind_group(a: number, b: number, c: number, d: number, e: number): void;
export function wgpu_compute_pass_set_push_constant(a: number, b: number, c: number, d: number): void;
export function wgpu_compute_pass_insert_debug_marker(a: number, b: number, c: number): void;
export function wgpu_compute_pass_push_debug_group(a: number, b: number, c: number): void;
export function wgpu_compute_pass_pop_debug_group(a: number): void;
export function wgpu_compute_pass_write_timestamp(a: number, b: number, c: number): void;
export function wgpu_compute_pass_begin_pipeline_statistics_query(a: number, b: number, c: number): void;
export function wgpu_compute_pass_end_pipeline_statistics_query(a: number): void;
export function wgpu_compute_pass_dispatch_workgroups(a: number, b: number, c: number, d: number): void;
export function wgpu_compute_pass_dispatch_workgroups_indirect(a: number, b: number, c: number): void;
export function wgpu_render_bundle_set_pipeline(a: number, b: number): void;
export function wgpu_render_bundle_set_bind_group(a: number, b: number, c: number, d: number, e: number): void;
export function wgpu_render_bundle_set_vertex_buffer(a: number, b: number, c: number, d: number, e: number): void;
export function wgpu_render_bundle_set_push_constants(a: number, b: number, c: number, d: number, e: number): void;
export function wgpu_render_bundle_draw(a: number, b: number, c: number, d: number, e: number): void;
export function wgpu_render_bundle_draw_indexed(a: number, b: number, c: number, d: number, e: number, f: number): void;
export function wgpu_render_bundle_draw_indirect(a: number, b: number, c: number): void;
export function wgpu_render_bundle_draw_indexed_indirect(a: number, b: number, c: number): void;
export function wgpu_render_pass_set_pipeline(a: number, b: number): void;
export function wgpu_render_pass_set_bind_group(a: number, b: number, c: number, d: number, e: number): void;
export function wgpu_render_pass_set_vertex_buffer(a: number, b: number, c: number, d: number, e: number): void;
export function wgpu_render_pass_set_push_constants(a: number, b: number, c: number, d: number, e: number): void;
export function wgpu_render_pass_draw(a: number, b: number, c: number, d: number, e: number): void;
export function wgpu_render_pass_draw_indexed(a: number, b: number, c: number, d: number, e: number, f: number): void;
export function wgpu_render_pass_draw_indirect(a: number, b: number, c: number): void;
export function wgpu_render_pass_draw_indexed_indirect(a: number, b: number, c: number): void;
export function wgpu_render_pass_multi_draw_indirect(a: number, b: number, c: number, d: number): void;
export function wgpu_render_pass_multi_draw_indexed_indirect(a: number, b: number, c: number, d: number): void;
export function wgpu_render_pass_multi_draw_indirect_count(a: number, b: number, c: number, d: number, e: number, f: number): void;
export function wgpu_render_pass_multi_draw_indexed_indirect_count(a: number, b: number, c: number, d: number, e: number, f: number): void;
export function wgpu_render_pass_set_blend_constant(a: number, b: number): void;
export function wgpu_render_pass_set_scissor_rect(a: number, b: number, c: number, d: number, e: number): void;
export function wgpu_render_pass_set_viewport(a: number, b: number, c: number, d: number, e: number, f: number, g: number): void;
export function wgpu_render_pass_set_stencil_reference(a: number, b: number): void;
export function wgpu_render_pass_insert_debug_marker(a: number, b: number, c: number): void;
export function wgpu_render_pass_push_debug_group(a: number, b: number, c: number): void;
export function wgpu_render_pass_pop_debug_group(a: number): void;
export function wgpu_render_pass_write_timestamp(a: number, b: number, c: number): void;
export function wgpu_render_pass_begin_occlusion_query(a: number, b: number): void;
export function wgpu_render_pass_end_occlusion_query(a: number): void;
export function wgpu_render_pass_begin_pipeline_statistics_query(a: number, b: number, c: number): void;
export function wgpu_render_pass_end_pipeline_statistics_query(a: number): void;
export function wgpu_render_pass_execute_bundles(a: number, b: number, c: number): void;
export function wgpu_render_bundle_set_index_buffer(a: number, b: number, c: number, d: number, e: number): void;
export function wgpu_render_bundle_pop_debug_group(a: number): void;
export function wgpu_render_bundle_insert_debug_marker(a: number, b: number): void;
export function wgpu_render_pass_set_index_buffer(a: number, b: number, c: number, d: number, e: number): void;
export function wgpu_render_bundle_push_debug_group(a: number, b: number): void;
export function __wbindgen_malloc(a: number, b: number): number;
export function __wbindgen_realloc(a: number, b: number, c: number, d: number): number;
export const __wbindgen_export_2: WebAssembly.Table;
export function wasm_bindgen__convert__closures__invoke1_mut__h4793a9a97d6451c0(a: number, b: number, c: number): void;
export function wasm_bindgen__convert__closures__invoke0_mut__hc6632823cce6b497(a: number, b: number): void;
export function wasm_bindgen__convert__closures__invoke2_mut__h0c96cacf276fcd00(a: number, b: number, c: number, d: number): void;
export function wasm_bindgen__convert__closures__invoke1_mut__h13e62b4bf399df80(a: number, b: number, c: number): void;
export function wasm_bindgen__convert__closures__invoke0_mut__hbb472b858c5d379e(a: number, b: number): void;
export function wasm_bindgen__convert__closures__invoke0_mut__hbd9702b266f4050a(a: number, b: number): void;
export function wasm_bindgen__convert__closures__invoke1_mut__hf79fd0f793df5ff6(a: number, b: number, c: number): void;
export function __wbindgen_free(a: number, b: number, c: number): void;
export function __wbindgen_exn_store(a: number): void;
export function __wbindgen_start(): void;
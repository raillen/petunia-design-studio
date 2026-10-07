# Batch Rendering in 2D

Batch rendering is a critical optimization technique in 2D rendering pipelines to minimize state changes and draw calls, which are expensive operations on the CPU-GPU boundary.

## The Problem

A naive rendering loop might look like this:
```cpp
for (auto& sprite : sprites) {
    bindTexture(sprite.texture);
    setShader(sprite.shader);
    drawQuad(sprite.position, sprite.size); // 1 Draw Call per sprite
}
```
If you have 10,000 sprites, this results in 10,000 draw calls per frame, which will severely bottleneck the CPU.

## The Solution: Sprite Batching

A Sprite Batcher accumulates geometry (vertices, colors, texture coordinates) into a large dynamically sized vertex buffer. It only issues a draw call when:
1. The buffer is full.
2. A state change is required (e.g., switching to a different texture atlas or shader).
3. The frame ends (flushing the remaining geometry).

### Texture Atlases
To maximize batching efficiency, individual sprite textures are packed into a single large texture called a Texture Atlas or Sprite Sheet. This allows multiple different sprites to be drawn with the same texture binding state, preventing premature batch flushes.

### Dynamic VBOs
The VBO (Vertex Buffer Object) used for batching should be mapped or updated using `glBufferSubData` (in OpenGL) or ring buffers to avoid CPU-GPU synchronization stalls.

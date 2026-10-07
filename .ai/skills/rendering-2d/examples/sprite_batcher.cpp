#include <vector>
#include <array>
#include <cstdint>

struct Vertex {
    float x, y;
    float u, v;
    uint32_t color;
};

class SpriteBatcher {
private:
    static constexpr size_t MAX_SPRITES = 10000;
    static constexpr size_t MAX_VERTICES = MAX_SPRITES * 4;
    static constexpr size_t MAX_INDICES = MAX_SPRITES * 6;

    std::vector<Vertex> vertices;
    uint32_t currentTextureID = 0;
    size_t spriteCount = 0;

public:
    SpriteBatcher() {
        vertices.reserve(MAX_VERTICES);
    }

    void begin() {
        vertices.clear();
        spriteCount = 0;
        currentTextureID = 0; // 0 means uninitialized
    }

    void drawSprite(float x, float y, float w, float h, uint32_t textureID) {
        if (spriteCount >= MAX_SPRITES || (currentTextureID != textureID && currentTextureID != 0)) {
            flush();
        }

        currentTextureID = textureID;

        // Add 4 vertices for the quad
        vertices.push_back({x, y, 0.0f, 0.0f, 0xFFFFFFFF});
        vertices.push_back({x + w, y, 1.0f, 0.0f, 0xFFFFFFFF});
        vertices.push_back({x + w, y + h, 1.0f, 1.0f, 0xFFFFFFFF});
        vertices.push_back({x, y + h, 0.0f, 1.0f, 0xFFFFFFFF});

        spriteCount++;
    }

    void flush() {
        if (spriteCount == 0) return;

        // In a real engine, we would upload `vertices.data()` to a VBO 
        // and issue a draw call using the bound `currentTextureID`.
        // bindTexture(currentTextureID);
        // updateDynamicVBO(vertices.data(), vertices.size() * sizeof(Vertex));
        // drawElements(GL_TRIANGLES, spriteCount * 6, GL_UNSIGNED_INT, nullptr);

        vertices.clear();
        spriteCount = 0;
    }

    void end() {
        flush();
    }
};

int main() {
    SpriteBatcher batcher;
    batcher.begin();
    batcher.drawSprite(10, 10, 32, 32, 1);
    batcher.drawSprite(50, 10, 32, 32, 1);
    batcher.drawSprite(100, 10, 32, 32, 2); // This will cause a flush
    batcher.end();
    return 0;
}

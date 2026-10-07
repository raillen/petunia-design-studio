#pragma once

#include <cmath>
#include <stdexcept>

namespace petunia::core {

class NonFiniteError : public std::domain_error {
public:
    using std::domain_error::domain_error;
};

inline void require_finite(double v, const char* what) {
    if (!std::isfinite(v)) throw NonFiniteError(what);
}

struct Vec2d {
    double x = 0.0;
    double y = 0.0;
    Vec2d operator+(Vec2d o) const { return {x + o.x, y + o.y}; }
    Vec2d operator-(Vec2d o) const { return {x - o.x, y - o.y}; }
    Vec2d operator*(double s) const { return {x * s, y * s}; }
    bool operator==(const Vec2d&) const = default;
    void validate() const { require_finite(x, "vec.x"); require_finite(y, "vec.y"); }
};

struct Size {
    double width = 0.0;
    double height = 0.0;
    bool operator==(const Size&) const = default;
    void validate() const {
        require_finite(width, "size.width");
        require_finite(height, "size.height");
        if (width < 0 || height < 0) throw std::domain_error("negative size");
    }
};

struct Insets {
    double left = 0, top = 0, right = 0, bottom = 0;
    bool operator==(const Insets&) const = default;
};

struct Rectd {
    double x = 0, y = 0, width = 0, height = 0;
    bool operator==(const Rectd&) const = default;
    void validate() const {
        require_finite(x, "rect.x"); require_finite(y, "rect.y");
        require_finite(width, "rect.width"); require_finite(height, "rect.height");
    }
    Rectd inset(Insets in) const {
        return {x + in.left, y + in.top, width - in.left - in.right, height - in.top - in.bottom};
    }
};

// 2D affine: | a c tx |
//            | b d ty |
struct Affine2D {
    double a = 1, b = 0, c = 0, d = 1, tx = 0, ty = 0;

    static Affine2D translation(double dx, double dy) { return {1, 0, 0, 1, dx, dy}; }
    static Affine2D scale(double sx, double sy) { return {sx, 0, 0, sy, 0, 0}; }

    Vec2d apply(Vec2d p) const { return {a * p.x + c * p.y + tx, b * p.x + d * p.y + ty}; }

    Affine2D then(Affine2D o) const {
        return {a * o.a + c * o.b, b * o.a + d * o.b,
                a * o.c + c * o.d, b * o.c + d * o.d,
                a * o.tx + c * o.ty + tx, b * o.tx + d * o.ty + ty};
    }

    Affine2D inverted() const {
        const double det = a * d - b * c;
        if (det == 0.0) throw std::domain_error("singular transform");
        const double inv = 1.0 / det;
        return {d * inv, -b * inv, -c * inv, a * inv,
                (c * ty - d * tx) * inv, (b * tx - a * ty) * inv};
    }

    bool operator==(const Affine2D&) const = default;
};

inline double degrees_to_radians(double deg) {
    return deg * 3.14159265358979323846 / 180.0;
}
inline double radians_to_degrees(double rad) {
    return rad * 180.0 / 3.14159265358979323846;
}

} // namespace petunia::core

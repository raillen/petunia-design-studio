#pragma once

#include <array>
#include <charconv>
#include <concepts>
#include <cstdint>
#include <format>
#include <functional>
#include <random>
#include <span>
#include <stdexcept>
#include <string>
#include <string_view>
#include <type_traits>

namespace petunia::core {

class InvalidIdError : public std::invalid_argument {
public:
    using std::invalid_argument::invalid_argument;
};

// 128-bit stable identifier value. Never derived from pointers or indices.
class Uuid {
public:
    Uuid() = default;
    explicit Uuid(std::array<std::uint8_t, 16> bytes) : bytes_(bytes) {}

    static Uuid generate() {
        static thread_local std::mt19937_64 rng{std::random_device{}()};
        std::array<std::uint8_t, 16> b{};
        for (std::size_t i = 0; i < 2; ++i) {
            const auto v = rng();
            for (std::size_t j = 0; j < 8; ++j) b[i * 8 + j] = static_cast<std::uint8_t>(v >> (j * 8));
        }
        return Uuid{b};
    }

    static Uuid parse(std::string_view text) {
        // Accepts 32 hex chars, optionally with '-' separators (8-4-4-4-12).
        std::array<std::uint8_t, 16> b{};
        std::size_t out = 0;
        for (char c : text) {
            if (c == '-') continue;
            if (out >= 32) throw InvalidIdError("uuid too long");
            hex_[out++] = c;
        }
        if (out != 32) throw InvalidIdError("uuid must have 32 hex chars");
        for (std::size_t i = 0; i < 16; ++i) {
            unsigned v = 0;
            auto [ptr, ec] = std::from_chars(&hex_[i * 2], &hex_[i * 2 + 2], v, 16);
            if (ec != std::errc{} || ptr != &hex_[i * 2 + 2]) throw InvalidIdError("bad hex");
            b[i] = static_cast<std::uint8_t>(v);
        }
        return Uuid{b};
    }

    std::string str() const {
        static constexpr char kHex[] = "0123456789abcdef";
        std::string s;
        s.reserve(36);
        for (std::size_t i = 0; i < 16; ++i) {
            if (i == 4 || i == 6 || i == 8 || i == 10) s.push_back('-');
            s.push_back(kHex[bytes_[i] >> 4]);
            s.push_back(kHex[bytes_[i] & 0xF]);
        }
        return s;
    }

    const std::array<std::uint8_t, 16>& bytes() const { return bytes_; }
    bool operator==(const Uuid&) const = default;
    auto operator<=>(const Uuid&) const = default;

private:
    std::array<std::uint8_t, 16> bytes_{};
    static thread_local char hex_[32];
};

inline thread_local char Uuid::hex_[32];

template <class Tag>
class TypedId {
public:
    TypedId() = default;
    explicit TypedId(Uuid id) : id_(id) {}

    static TypedId generate() { return TypedId{Uuid::generate()}; }
    static TypedId parse(std::string_view text) { return TypedId{Uuid::parse(text)}; }
    std::string str() const { return id_.str(); }
    const Uuid& uuid() const { return id_; }

    bool operator==(const TypedId&) const = default;
    auto operator<=>(const TypedId&) const = default;

private:
    Uuid id_{};
};

namespace tags {
struct Document;
struct Object;
struct Surface;
struct Resource;
struct Style;
struct Symbol;
struct Story;
struct Node;
struct Transaction;
struct Job;
} // namespace tags

using DocumentId = TypedId<tags::Document>;
using ObjectId = TypedId<tags::Object>;
using SurfaceId = TypedId<tags::Surface>;
using ResourceId = TypedId<tags::Resource>;
using StyleId = TypedId<tags::Style>;
using SymbolId = TypedId<tags::Symbol>;
using StoryId = TypedId<tags::Story>;
using NodeId = TypedId<tags::Node>;
using TransactionId = TypedId<tags::Transaction>;
using JobId = TypedId<tags::Job>;

} // namespace petunia::core

template <class Tag>
struct std::hash<petunia::core::TypedId<Tag>> {
    std::size_t operator()(const petunia::core::TypedId<Tag>& id) const noexcept {
        std::size_t h = 1469598103934665603ull;
        for (std::uint8_t b : id.uuid().bytes()) {
            h ^= b;
            h *= 1099511628211ull;
        }
        return h;
    }
};

template <>
struct std::formatter<petunia::core::Uuid> : std::formatter<std::string> {
    auto format(const petunia::core::Uuid& id, std::format_context& ctx) const {
        return std::formatter<std::string>::format(id.str(), ctx);
    }
};

template <class Tag>
struct std::formatter<petunia::core::TypedId<Tag>> : std::formatter<std::string> {
    auto format(const petunia::core::TypedId<Tag>& id, std::format_context& ctx) const {
        return std::formatter<std::string>::format(id.str(), ctx);
    }
};

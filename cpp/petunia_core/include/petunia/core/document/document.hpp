#pragma once

#include <variant>
#include <optional>
#include <string>
#include <unordered_map>
#include <vector>

#include "petunia/core/error/error.hpp"
#include "petunia/core/ids/ids.hpp"
#include "petunia/core/math/types.hpp"

namespace petunia::core {

struct RectangleRecord {
    ObjectId id = ObjectId::generate();
    std::string name;
    Rectd frame{};
    bool visible = true;
};

struct GroupRecord {
    ObjectId id = ObjectId::generate();
    std::string name;
    std::vector<ObjectId> children;
    bool visible = true;
};

using ObjectRecord = std::variant<RectangleRecord, GroupRecord>;

struct Surface {
    SurfaceId id = SurfaceId::generate();
    std::string name;
    Size canvas{1000, 1000};
    std::vector<ObjectId> roots;
};

class DocumentStore {
public:
    DocumentStore() = default;

    SurfaceId add_surface(std::string name, Size canvas) {
        Surface s;
        s.name = std::move(name);
        s.canvas = canvas;
        const auto id = s.id;
        surfaces_.emplace(id, std::move(s));
        return id;
    }

    ObjectId add_rectangle(SurfaceId surface, std::optional<ObjectId> parent, RectangleRecord rec) {
        auto& s = require_surface(surface);
        const auto id = rec.id;
        objects_.emplace(id, std::move(rec));
        parents_[id] = parent;
        attach(surface, parent, id);
        (void)s;
        return id;
    }

    ObjectId add_group(SurfaceId surface, std::optional<ObjectId> parent, GroupRecord rec) {
        require_surface(surface);
        const auto id = rec.id;
        objects_.emplace(id, std::move(rec));
        parents_[id] = parent;
        attach(surface, parent, id);
        return id;
    }

    void reparent(ObjectId id, SurfaceId surface, std::optional<ObjectId> newParent) {
        auto it = parents_.find(id);
        if (it == parents_.end()) throw Error(ErrorCode::NotFound, "object not found");
        auto& s = require_surface(surface);
        const auto old = it->second;
        if (old) {
            if (auto* g = group(*old)) {
                std::erase(g->children, id);
            }
        } else {
            std::erase(s.roots, id);
        }
        // cycle check: newParent's ancestor chain must not contain id
        for (std::optional<ObjectId> p = newParent; p; p = parent(*p)) {
            if (*p == id) throw Error(ErrorCode::Cycle, "reparent would create cycle");
        }
        parents_[id] = newParent;
        if (newParent) {
            if (auto* g = group(*newParent)) g->children.push_back(id);
        } else {
            s.roots.push_back(id);
        }
    }

    void remove(ObjectId id) {
        auto it = parents_.find(id);
        if (it == parents_.end()) return;
        const auto oldParent = it->second;
        if (oldParent) {
            if (auto* g = group(*oldParent)) std::erase(g->children, id);
        } else {
            for (auto& [sid, s] : surfaces_) std::erase(s.roots, id);
        }
        parents_.erase(it);
        objects_.erase(id);
    }

    std::optional<ObjectId> parent(ObjectId id) const {
        auto it = parents_.find(id);
        return it == parents_.end() ? std::nullopt : it->second;
    }

    const ObjectRecord* find(ObjectId id) const {
        auto it = objects_.find(id);
        return it == objects_.end() ? nullptr : &it->second;
    }
    ObjectRecord* find(ObjectId id) {
        auto it = objects_.find(id);
        return it == objects_.end() ? nullptr : &it->second;
    }

    const Surface* find_surface(SurfaceId id) const {
        auto it = surfaces_.find(id);
        return it == surfaces_.end() ? nullptr : &it->second;
    }

    std::size_t object_count() const { return objects_.size(); }

    // Mutation service for command layer.
    friend class DocumentMutator;

private:
    Surface& require_surface(SurfaceId id) {
        auto it = surfaces_.find(id);
        if (it == surfaces_.end()) throw Error(ErrorCode::NotFound, "surface not found");
        return it->second;
    }

    void attach(SurfaceId surface, std::optional<ObjectId> parent, ObjectId id) {
        auto& s = require_surface(surface);
        if (parent) {
            if (auto* g = group(*parent)) g->children.push_back(id);
            else throw Error(ErrorCode::Validation, "parent is not a group");
        } else {
            s.roots.push_back(id);
        }
        surface_of_[id] = surface;
    }

    GroupRecord* group(ObjectId id) {
        auto* rec = find(id);
        return rec ? std::get_if<GroupRecord>(rec) : nullptr;
    }

    std::unordered_map<SurfaceId, Surface> surfaces_;
    std::unordered_map<ObjectId, ObjectRecord> objects_;
    std::unordered_map<ObjectId, std::optional<ObjectId>> parents_;
    std::unordered_map<ObjectId, SurfaceId> surface_of_;
};

} // namespace petunia::core

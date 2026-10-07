#pragma once

#include <memory>
#include <cstdint>
#include <functional>
#include <string>
#include <vector>

#include "petunia/core/document/document.hpp"
#include "petunia/core/ids/ids.hpp"

namespace petunia::core {

struct ChangeSet {
    std::vector<ObjectId> added;
    std::vector<ObjectId> removed;
    std::vector<ObjectId> modified;
    std::vector<ObjectId> reparented;
    std::uint64_t newRevision = 0;
};

struct CommandContext {
    DocumentStore& store;
    ChangeSet changes;
};

class Command {
public:
    virtual ~Command() = default;
    virtual std::string_view name() const = 0;
    virtual void apply(CommandContext& ctx) = 0;
    virtual void revert(CommandContext& ctx) = 0;
};

class Transaction {
public:
    explicit Transaction(DocumentStore& store) : store_(store) {}

    void add(std::unique_ptr<Command> cmd) { commands_.push_back(std::move(cmd)); }

    ChangeSet commit() {
        CommandContext ctx{store_, {}};
        std::size_t applied = 0;
        try {
            for (auto& c : commands_) {
                c->apply(ctx);
                ++applied;
            }
        } catch (...) {
            for (std::size_t i = applied; i > 0; --i) {
                try {
                    commands_[i - 1]->revert(ctx);
                } catch (...) {
                    // best-effort rollback
                }
            }
            throw;
        }
        ctx.changes.newRevision = ++revision_;
        return std::move(ctx.changes);
    }

    std::uint64_t revision() const { return revision_; }

    std::vector<std::unique_ptr<Command>> release() { return std::move(commands_); }

private:
    DocumentStore& store_;
    std::vector<std::unique_ptr<Command>> commands_;
    std::uint64_t revision_ = 0;
};

class History {
public:
    void push(std::vector<std::unique_ptr<Command>> cmds, ChangeSet cs) {
        undo_.push_back(std::move(cmds));
        changes_.push_back(std::move(cs));
        redo_.clear();
        redoChanges_.clear();
    }

    bool can_undo() const { return !undo_.empty(); }
    bool can_redo() const { return !redo_.empty(); }

    void undo(DocumentStore& store) {
        if (undo_.empty()) return;
        CommandContext ctx{store, {}};
        for (auto it = undo_.back().rbegin(); it != undo_.back().rend(); ++it) (*it)->revert(ctx);
        redo_.push_back(std::move(undo_.back()));
        redoChanges_.push_back(std::move(changes_.back()));
        undo_.pop_back();
        changes_.pop_back();
    }

    void redo(DocumentStore& store) {
        if (redo_.empty()) return;
        CommandContext ctx{store, {}};
        for (auto& c : redo_.back()) c->apply(ctx);
        undo_.push_back(std::move(redo_.back()));
        changes_.push_back(std::move(redoChanges_.back()));
        redo_.pop_back();
        redoChanges_.pop_back();
    }

private:
    std::vector<std::vector<std::unique_ptr<Command>>> undo_;
    std::vector<std::vector<std::unique_ptr<Command>>> redo_;
    std::vector<ChangeSet> changes_;
    std::vector<ChangeSet> redoChanges_;
};

struct TransformRectangleCommand : Command {
    explicit TransformRectangleCommand(ObjectId id, Rectd to) : id_(id), to_(to) {}
    std::string_view name() const override { return "transform-rectangle"; }
    void apply(CommandContext& ctx) override {
        auto* rec = ctx.store.find(id_);
        if (!rec) throw Error(ErrorCode::NotFound, "object not found");
        auto& r = std::get<RectangleRecord>(*rec);
        from_ = r.frame;
        r.frame = to_;
        ctx.changes.modified.push_back(id_);
    }
    void revert(CommandContext& ctx) override {
        auto* rec = ctx.store.find(id_);
        if (rec) std::get<RectangleRecord>(*rec).frame = from_;
    }

private:
    ObjectId id_;
    Rectd to_;
    Rectd from_{};
};

} // namespace petunia::core

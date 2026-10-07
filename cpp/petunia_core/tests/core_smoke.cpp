#include <cassert>
#include <cmath>
#include <atomic>
#include <chrono>
#include <iostream>
#include <thread>

#include "petunia/core/command/command.hpp"
#include "petunia/core/document/document.hpp"
#include "petunia/core/ids/ids.hpp"
#include "petunia/core/jobs/jobs.hpp"
#include "petunia/core/math/types.hpp"

namespace pc = petunia::core;

int main() {
    // IDs roundtrip
    [[maybe_unused]] auto id = pc::ObjectId::generate();
    assert(pc::ObjectId::parse(id.str()) == id);

    // Math invariants
    pc::Affine2D t = pc::Affine2D::translation(3, 4);
    pc::Vec2d p{1, 2};
    [[maybe_unused]] auto back = t.inverted().apply(t.apply(p));
    assert(std::abs(back.x - 1) < 1e-9 && std::abs(back.y - 2) < 1e-9);
    auto composed = pc::Affine2D::translation(2, 0).then(pc::Affine2D::scale(2, 2));
    [[maybe_unused]] auto q = composed.apply(pc::Vec2d{1, 1});
    assert(std::abs(q.x - 4) < 1e-9 && std::abs(q.y - 2) < 1e-9);

    // Document + command + undo
    pc::DocumentStore store;
    auto sid = store.add_surface("S", {100, 100});
    pc::RectangleRecord r;
    r.name = "rect";
    r.frame = {0, 0, 10, 10};
    auto rid = store.add_rectangle(sid, std::nullopt, r);

    pc::Transaction tx(store);
    tx.add(std::make_unique<pc::TransformRectangleCommand>(rid, pc::Rectd{5, 5, 20, 20}));
    auto cs = tx.commit();
    auto rec = std::get<pc::RectangleRecord>(*store.find(rid));
    assert(rec.frame.x == 5 && cs.modified.size() == 1);

    pc::History h;
    h.push(tx.release(), std::move(cs));
    h.undo(store);
    rec = std::get<pc::RectangleRecord>(*store.find(rid));
    assert(rec.frame.x == 0);
    h.redo(store);
    rec = std::get<pc::RectangleRecord>(*store.find(rid));
    assert(rec.frame.x == 5);

    // Jobs
    pc::JobScheduler sched(2, 8);
    std::atomic done{0};
    pc::JobHandle* h1 = sched.submit(pc::JobPriority::High, [&](std::stop_token) { ++done; });
    for (int i = 0; i < 100 && h1->state.load() != pc::JobState::Succeeded; ++i)
        std::this_thread::sleep_for(std::chrono::milliseconds(10));
    assert(done == 1);

    std::cout << "core_smoke ok\n";
    return 0;
}

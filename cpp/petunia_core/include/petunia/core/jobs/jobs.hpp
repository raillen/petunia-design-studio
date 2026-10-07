#pragma once

#include <atomic>
#include <condition_variable>
#include <deque>
#include <functional>
#include <mutex>
#include <optional>
#include <stop_token>
#include <thread>
#include <memory>
#include <algorithm>
#include <stop_token>
#include <vector>

#include "petunia/core/error/error.hpp"
#include "petunia/core/ids/ids.hpp"

namespace petunia::core {

enum class JobPriority { Low = 0, Normal = 1, High = 2 };
enum class JobState { Queued, Running, Succeeded, Failed, Cancelled };

struct JobHandle {
    explicit JobHandle(JobId id_) : id(id_) {}
    JobId id;
    std::atomic<JobState> state{JobState::Queued};
    std::atomic<float> progress{0.0f};
    std::atomic_bool cancelRequested{false};
};

class JobScheduler {
public:
    explicit JobScheduler(std::size_t workers, std::size_t maxQueue = 256)
        : maxQueue_(maxQueue) {
        for (std::size_t i = 0; i < workers; ++i)
            workers_.emplace_back([this](std::stop_token st) { loop(st); });
    }

    ~JobScheduler() { shutdown(); }

    JobHandle* submit(JobPriority prio, std::function<void(std::stop_token)> fn) {
        std::lock_guard lock(mu_);
        if (queue_.size() >= maxQueue_) throw Error(ErrorCode::Validation, "job queue full");
        auto handle = std::make_unique<JobHandle>(JobId::generate());
        handle->state = JobState::Queued;
        auto* raw = handle.get();
        auto source = std::make_shared<std::stop_source>();
        sources_.push_back({raw->id, source});
        queue_.push_back({std::move(handle), std::move(fn), prio, source});
        std::sort(queue_.begin(), queue_.end(),
                  [](const Entry& a, const Entry& b) { return a.prio > b.prio; });
        cv_.notify_one();
        return raw;
    }

    void shutdown() {
        for (auto& w : workers_) w.request_stop();
        cv_.notify_all();
        workers_.clear();
    }

    void cancel(JobHandle* h) {
        h->cancelRequested = true;
        std::lock_guard lock(mu_);
        for (auto& e : sources_) {
            if (e.first == h->id) e.second->request_stop();
        }
    }

private:
    struct Entry {
        std::unique_ptr<JobHandle> handle;
        std::function<void(std::stop_token)> fn;
        JobPriority prio;
        std::shared_ptr<std::stop_source> source;
    };

    void loop(std::stop_token st) {
        while (true) {
            Entry e;
            {
                std::unique_lock lock(mu_);
                cv_.wait(lock, [&] { return !queue_.empty() || st.stop_requested(); });
                if (st.stop_requested() && queue_.empty()) return;
                e = std::move(queue_.front());
                queue_.pop_front();
            }
            e.handle->state = JobState::Running;
            try {
                e.fn(e.source->get_token());
                e.handle->state = e.handle->cancelRequested ? JobState::Cancelled : JobState::Succeeded;
            } catch (const std::exception&) {
                e.handle->state = JobState::Failed;
            }
            e.handle->progress = 1.0f;
            std::lock_guard lock(mu_);
            std::erase_if(sources_, [&](auto& p) { return p.first == e.handle->id; });
            done_.push_back(std::move(e.handle));
        }
    }

    std::mutex mu_;
    std::condition_variable cv_;
    std::deque<Entry> queue_;
    std::vector<std::pair<JobId, std::shared_ptr<std::stop_source>>> sources_;
    std::vector<std::unique_ptr<JobHandle>> done_;
    std::vector<std::jthread> workers_;
    std::size_t maxQueue_;
};

} // namespace petunia::core

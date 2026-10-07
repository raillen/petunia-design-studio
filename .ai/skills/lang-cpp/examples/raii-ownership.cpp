#include <iostream>
#include <memory>
#include <span>
#include <vector>
#include <string_view>

class SafeResource {
public:
    explicit SafeResource(std::string_view name)
        : name_(name) {
        std::cout << "Resource acquired: " << name_ << '\n';
    }

    ~SafeResource() {
        std::cout << "Resource safely released: " << name_ << '\n';
    }

    // Move-only semantics (no accidental copying)
    SafeResource(const SafeResource&) = delete;
    SafeResource& operator=(const SafeResource&) = delete;
    SafeResource(SafeResource&&) noexcept = default;
    SafeResource& operator=(SafeResource&&) noexcept = default;

    void process(std::span<const int> data) const {
        for (int val : data) {
            std::cout << "Value: " << val << '\n';
        }
    }

private:
    std::string name_;
};

int main() {
    auto res = std::make_unique<SafeResource>("DatabaseSession");
    const std::vector<int> numbers = {10, 20, 30};
    res->process(numbers);
    return 0;
}

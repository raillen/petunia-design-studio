#pragma once

#include <cstdint>
#include <optional>
#include <stdexcept>
#include <string>
#include <utility>

namespace petunia::core {

enum class ErrorCode : std::uint16_t {
    InvalidArgument = 1000,
    NonFinite = 1001,
    NotFound = 1002,
    AlreadyExists = 1003,
    Cycle = 1004,
    Validation = 1005,
    Cancelled = 2000,
    Timeout = 2001,
    Io = 3000,
    Format = 3001,
    Internal = 9000,
};

enum class Severity { Recoverable, Fatal };

class Error : public std::runtime_error {
public:
    Error(ErrorCode code, std::string message, Severity severity = Severity::Recoverable)
        : std::runtime_error(std::move(message)), code_(code), severity_(severity) {}

    ErrorCode code() const noexcept { return code_; }
    Severity severity() const noexcept { return severity_; }

    std::optional<std::string> path;
    std::optional<std::string> objectId;
    std::optional<std::string> correlationId;

private:
    ErrorCode code_;
    Severity severity_;
};

template <class T>
class Result {
public:
    Result(T value) : value_(std::move(value)) {}
    Result(Error error) : error_(std::move(error)) {}

    bool ok() const noexcept { return value_.has_value(); }
    const T& value() const {
        if (!value_) throw *error_;
        return *value_;
    }
    T& value() {
        if (!value_) throw *error_;
        return *value_;
    }
    const Error& error() const {
        if (!error_) throw std::logic_error("no error");
        return *error_;
    }

private:
    std::optional<T> value_;
    std::optional<Error> error_;
};

template <>
class Result<void> {
public:
    Result() = default;
    Result(Error error) : error_(std::move(error)) {}
    bool ok() const noexcept { return !error_.has_value(); }
    void value() const {
        if (error_) throw *error_;
    }
    const Error& error() const {
        if (!error_) throw std::logic_error("no error");
        return *error_;
    }

private:
    std::optional<Error> error_;
};

} // namespace petunia::core

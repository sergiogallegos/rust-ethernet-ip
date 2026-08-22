#ifndef RUST_ETHERNET_IP_CPP_EIP_CLIENT_HPP
#define RUST_ETHERNET_IP_CPP_EIP_CLIENT_HPP

#include "rust_ethernet_ip.h"

#include <array>
#include <cmath>
#include <cstdint>
#include <stdexcept>
#include <string>
#include <utility>
#include <vector>

namespace rust_ethernet_ip {

struct Error {
    int code = 0;
    std::string message;
};

template <typename T>
struct Result {
    bool ok = false;
    T value {};
    Error error {};

    explicit operator bool() const { return ok; }
};

template <>
struct Result<void> {
    bool ok = false;
    Error error {};

    explicit operator bool() const { return ok; }
};

class EipClient {
public:
    EipClient() = default;

    explicit EipClient(int client_id) : client_id_(client_id) {}

    EipClient(const EipClient &) = delete;
    EipClient &operator=(const EipClient &) = delete;

    EipClient(EipClient &&other) noexcept : client_id_(std::exchange(other.client_id_, 0)) {}

    EipClient &operator=(EipClient &&other) noexcept
    {
        if (this != &other) {
            close();
            client_id_ = std::exchange(other.client_id_, 0);
        }
        return *this;
    }

    ~EipClient() { close(); }

    static Result<EipClient> connect(const std::string &address)
    {
        const int id = eip_connect(address.c_str());
        if (id <= 0) {
            return failure<EipClient>(id, read_last_error(id));
        }
        return success(EipClient(id));
    }

    int id() const { return client_id_; }

    Result<void> write_dint(const std::string &tag, int value)
    {
        return check(eip_write_dint(client_id_, tag.c_str(), value));
    }

    Result<int> read_dint(const std::string &tag)
    {
        int value = 0;
        const int rc = eip_read_dint(client_id_, tag.c_str(), &value);
        if (rc != 0) {
            return failure<int>(rc, last_error());
        }
        return success(value);
    }

    Result<void> write_real(const std::string &tag, double value)
    {
        return check(eip_write_real(client_id_, tag.c_str(), value));
    }

    Result<double> read_real(const std::string &tag)
    {
        double value = 0.0;
        const int rc = eip_read_real(client_id_, tag.c_str(), &value);
        if (rc != 0) {
            return failure<double>(rc, last_error());
        }
        return success(value);
    }

    Result<void> write_string(const std::string &tag, const std::string &value)
    {
        return check(eip_write_string(client_id_, tag.c_str(), value.c_str()));
    }

    Result<std::string> read_string(const std::string &tag)
    {
        std::array<char, 512> buffer {};
        const int rc = eip_read_string(client_id_, tag.c_str(), buffer.data(), static_cast<int>(buffer.size()));
        if (rc != 0) {
            return failure<std::string>(rc, last_error());
        }
        return success(std::string(buffer.data()));
    }

    Result<std::string> read_tags_batch(const std::vector<std::string> &tags)
    {
        std::vector<const char *> names;
        names.reserve(tags.size());
        for (const auto &tag : tags) {
            names.push_back(tag.c_str());
        }
        std::array<char, 4096> buffer {};
        const int rc = eip_read_tags_batch(
            client_id_,
            names.data(),
            static_cast<int>(names.size()),
            buffer.data(),
            static_cast<int>(buffer.size()));
        if (rc != 0) {
            return failure<std::string>(rc, last_error());
        }
        return success(std::string(buffer.data()));
    }

    Result<std::string> write_tags_batch(const std::string &json_payload, int tag_count)
    {
        std::array<char, 4096> buffer {};
        const int rc = eip_write_tags_batch(
            client_id_,
            json_payload.c_str(),
            tag_count,
            buffer.data(),
            static_cast<int>(buffer.size()));
        if (rc != 0) {
            return failure<std::string>(rc, last_error());
        }
        return success(std::string(buffer.data()));
    }

    Result<std::string> execute_batch(const std::string &json_payload, int operation_count)
    {
        std::array<char, 4096> buffer {};
        const int rc = eip_execute_batch(
            client_id_,
            json_payload.c_str(),
            operation_count,
            buffer.data(),
            static_cast<int>(buffer.size()));
        if (rc != 0) {
            return failure<std::string>(rc, last_error());
        }
        return success(std::string(buffer.data()));
    }

    std::string last_error() const { return read_last_error(client_id_); }

    Result<void> refreshSchema()
    {
        return check(eip_refresh_schema(client_id_));
    }

    Result<std::string> diagnosticsJson(bool detailed = false)
    {
        char *json = nullptr;
        const int rc = eip_get_diagnostics_json(client_id_, detailed ? 1 : 0, &json);
        if (rc != 0 || json == nullptr) {
            return failure<std::string>(rc, last_error());
        }
        const std::string value(json);
        eip_free_string(json);
        return success(value);
    }

private:
    int client_id_ = 0;

    static std::string read_last_error(int client_id)
    {
        std::array<char, 1024> buffer {};
        const int written = eip_get_last_error(client_id, buffer.data(), static_cast<int>(buffer.size()));
        if (written <= 0) {
            return {};
        }
        return std::string(buffer.data(), static_cast<std::size_t>(written));
    }

    Result<void> check(int rc)
    {
        if (rc == 0) {
            return Result<void> { true, {} };
        }
        return Result<void> { false, Error { rc, last_error() } };
    }

    template <typename T>
    static Result<T> success(T value)
    {
        return Result<T> { true, std::move(value), {} };
    }

    template <typename T>
    static Result<T> failure(int code, std::string message)
    {
        return Result<T> { false, {}, Error { code, std::move(message) } };
    }

    void close()
    {
        if (client_id_ > 0) {
            eip_disconnect(client_id_);
            client_id_ = 0;
        }
    }
};

inline void require(bool condition, const std::string &message)
{
    if (!condition) {
        throw std::runtime_error(message);
    }
}

inline void require_ok(const Result<void> &result, const std::string &operation)
{
    if (!result.ok) {
        throw std::runtime_error(operation + " failed: " + result.error.message);
    }
}

template <typename T>
inline T require_value(const Result<T> &result, const std::string &operation)
{
    if (!result.ok) {
        throw std::runtime_error(operation + " failed: " + result.error.message);
    }
    return result.value;
}

} // namespace rust_ethernet_ip

#endif

// C/C++ full-coverage exerciser — parity with the Rust/C#/Python runners.
//
// Parses examples/full_coverage_tags.json directly (single source of truth),
// expands the categories with the same rules as examples/test_plc_full_coverage.rs,
// and drives the full read / write / verify / blocked-probe / settle surface
// through the C ABI in include/rust_ethernet_ip.h.
//
// Exercises STRING writes through the same manifest-driven phases as the
// Rust/C#/Python runners. After handle-aware STRING writes, the shared manifest
// resolves to 2304 read / 2285 write / 0 blocked / 19 read-only.
//
// Usage: full_coverage [--plc-address <ip:port>] [--plc-slot <n>]
//                      [--manifest <path>] [--out-dir <dir>] [--skip-preflight]
// Env fallbacks: TEST_PLC_ADDRESS (default 192.168.0.1:44818), TEST_PLC_SLOT (0).

#include "rust_ethernet_ip.h"

#include <algorithm>
#include <cctype>
#include <chrono>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <ctime>
#include <fstream>
#include <filesystem>
#include <iomanip>
#include <map>
#include <sstream>
#include <string>
#include <utility>
#include <vector>

// ------------------------------- tiny JSON ---------------------------------
struct Json {
    enum Type { Null, Bool, Num, Str, Arr, Obj } type = Null;
    bool b = false;
    double num = 0;
    std::string str;
    std::vector<Json> arr;
    std::vector<std::pair<std::string, Json>> obj; // insertion order preserved

    const Json *find(const std::string &k) const {
        if (type != Obj) return nullptr;
        for (const auto &p : obj)
            if (p.first == k) return &p.second;
        return nullptr;
    }
};

struct JsonParser {
    const char *p;
    const char *end;
    explicit JsonParser(const std::string &s) : p(s.data()), end(s.data() + s.size()) {}

    void ws() {
        while (p < end && (*p == ' ' || *p == '\t' || *p == '\n' || *p == '\r')) ++p;
    }
    Json parse() {
        ws();
        Json v = value();
        return v;
    }
    Json value() {
        ws();
        if (p >= end) return {};
        char c = *p;
        if (c == '{') return object();
        if (c == '[') return array();
        if (c == '"') { Json v; v.type = Json::Str; v.str = str(); return v; }
        if (c == 't' || c == 'f') return boolean();
        if (c == 'n') { p += 4; return {}; }
        return number();
    }
    std::string str() {
        std::string out;
        ++p; // opening quote
        while (p < end && *p != '"') {
            if (*p == '\\' && p + 1 < end) {
                ++p;
                switch (*p) {
                    case 'n': out += '\n'; break;
                    case 't': out += '\t'; break;
                    case '"': out += '"'; break;
                    case '\\': out += '\\'; break;
                    case '/': out += '/'; break;
                    default: out += *p; break;
                }
                ++p;
            } else {
                out += *p++;
            }
        }
        if (p < end) ++p; // closing quote
        return out;
    }
    Json number() {
        Json v;
        v.type = Json::Num;
        const char *start = p;
        while (p < end && (std::isdigit((unsigned char)*p) || *p == '-' || *p == '+' ||
                           *p == '.' || *p == 'e' || *p == 'E'))
            ++p;
        v.num = std::strtod(std::string(start, p).c_str(), nullptr);
        return v;
    }
    Json boolean() {
        Json v;
        v.type = Json::Bool;
        if (*p == 't') { v.b = true; p += 4; } else { v.b = false; p += 5; }
        return v;
    }
    Json array() {
        Json v;
        v.type = Json::Arr;
        ++p; // [
        ws();
        while (p < end && *p != ']') {
            v.arr.push_back(value());
            ws();
            if (p < end && *p == ',') { ++p; ws(); }
        }
        if (p < end) ++p; // ]
        return v;
    }
    Json object() {
        Json v;
        v.type = Json::Obj;
        ++p; // {
        ws();
        while (p < end && *p != '}') {
            ws();
            std::string key = str();
            ws();
            if (p < end && *p == ':') ++p;
            v.obj.emplace_back(std::move(key), value());
            ws();
            if (p < end && *p == ',') { ++p; ws(); }
        }
        if (p < end) ++p; // }
        return v;
    }
};

// ------------------------------- model -------------------------------------
enum class Kind { Dint, Int, Real, Bool, String, Udt };
enum class Write { Writeable, ReadOnly, BlockedString };

static Kind kind_of(const std::string &s) {
    if (s == "Dint") return Kind::Dint;
    if (s == "Int") return Kind::Int;
    if (s == "Real") return Kind::Real;
    if (s == "Bool") return Kind::Bool;
    if (s == "String") return Kind::String;
    return Kind::Udt;
}
static Write write_of(const std::string &s) {
    if (s == "writeable" || s == "service_layer_writeable") return Write::Writeable;
    if (s == "encoding_blocked_udt_string_member") return Write::BlockedString;
    return Write::ReadOnly;
}
static bool is_writeable(Write w) { return w == Write::Writeable; }
static bool is_blocked(Write w) { return w == Write::BlockedString; }

struct Tag {
    std::string name;
    std::string category;
    Kind kind;
    Write write;
};

static std::string render(std::string pat, long i, const std::string &member,
                          const std::string &field, long j) {
    auto replace = [&](const std::string &from, const std::string &to) {
        size_t pos;
        while ((pos = pat.find(from)) != std::string::npos) pat.replace(pos, from.size(), to);
    };
    if (i >= 0) replace("{i}", std::to_string(i));
    if (!member.empty()) replace("{member}", member);
    if (!field.empty()) replace("{field}", field);
    if (j >= 0) replace("{j}", std::to_string(j));
    return pat;
}

static std::vector<long> range_or_once(const Json *spec) {
    std::vector<long> out;
    if (spec) {
        const Json *r = spec->find("range");
        if (r && r->type == Json::Arr && r->arr.size() == 2) {
            long a = (long)r->arr[0].num, b = (long)r->arr[1].num;
            for (long i = a; i < b; ++i) out.push_back(i);
            return out;
        }
    }
    out.push_back(0);
    return out;
}

static void expand(const Json &cat, std::vector<Tag> &tags) {
    std::string name = cat.find("name")->str;
    std::string pattern = cat.find("pattern")->str;

    if (const Json *members = cat.find("members")) {
        for (long i : range_or_once(cat.find("indices"))) {
            for (const auto &m : members->obj) {
                tags.push_back({render(pattern, i, m.first, "", -1), name,
                                kind_of(m.second.find("kind")->str),
                                write_of(m.second.find("writeability")->str)});
            }
        }
        return;
    }
    if (const Json *inner = cat.find("inner")) {
        for (long i : range_or_once(cat.find("outer_indices"))) {
            for (const auto &f : inner->obj) {
                const Json *r = f.second.find("range");
                long a = (long)r->arr[0].num, b = (long)r->arr[1].num;
                for (long j = a; j < b; ++j) {
                    tags.push_back({render(pattern, i, "", f.first, j), name,
                                    kind_of(f.second.find("kind")->str),
                                    write_of(f.second.find("writeability")->str)});
                }
            }
        }
        return;
    }
    Kind k = kind_of(cat.find("kind")->str);
    Write w = write_of(cat.find("writeability")->str);
    for (long i : range_or_once(cat.find("indices")))
        tags.push_back({render(pattern, i, "", "", -1), name, k, w});
}

// ------------------------------- stats -------------------------------------
struct CatStats {
    long read_ok = 0, read_fail = 0, write_ok = 0, write_fail = 0;
    long verify_ok = 0, verify_fail = 0, blocked_ok = 0, blocked_unexpected = 0;
};

// A written value we can verify later, tagged by kind.
struct Written {
    size_t idx;
    Kind kind;
    int i = 0;
    double r = 0;
    std::string s;
};

// ------------------------------- IO helpers --------------------------------
static bool read_ok(int cid, const Tag &t) {
    char buf[8192];
    return eip_read_tag(cid, t.name.c_str(), buf, (int)sizeof(buf)) == 0;
}

static int write_terminal(int cid, const Tag &t) {
    switch (t.kind) {
        case Kind::Dint: return eip_write_dint(cid, t.name.c_str(), 999999);
        case Kind::Int: return eip_write_int(cid, t.name.c_str(), (int16_t)9999);
        case Kind::Bool: return eip_write_bool(cid, t.name.c_str(), 1);
        case Kind::Real: return eip_write_real(cid, t.name.c_str(), 99.99);
        case Kind::String: return eip_write_string(cid, t.name.c_str(), "SETTLED");
        case Kind::Udt: return -1;
    }
    return -1;
}

struct LatencyStats {
    size_t samples = 0;
    long failures = 0;
    double total_ms = 0, avg_ms = 0, min_ms = 0, p50_ms = 0, p95_ms = 0,
           p99_ms = 0, max_ms = 0, ops_per_sec = 0, tags_per_sec = 0,
           outlier_filtered_avg_ms = 0;
    size_t outlier_count = 0;
};

static LatencyStats summarize(std::vector<double> samples, long failures, size_t tags_per_batch = 1) {
    std::sort(samples.begin(), samples.end());
    LatencyStats s;
    s.samples = samples.size();
    s.failures = failures;
    for (double value : samples) s.total_ms += value;
    auto percentile = [&](double fraction) {
        if (samples.empty()) return 0.0;
        return samples[(size_t)std::llround((samples.size() - 1) * fraction)];
    };
    if (!samples.empty()) {
        s.avg_ms = s.total_ms / samples.size();
        s.min_ms = samples.front();
        s.p50_ms = percentile(0.50);
        s.p95_ms = percentile(0.95);
        s.p99_ms = percentile(0.99);
        s.max_ms = samples.back();
        if (s.total_ms > 0) {
            s.ops_per_sec = samples.size() * 1000.0 / s.total_ms;
            s.tags_per_sec = s.ops_per_sec * tags_per_batch;
        }
        const double q1 = percentile(0.25), q3 = percentile(0.75), iqr = q3 - q1;
        const double lower = q1 - 1.5 * iqr, upper = q3 + 1.5 * iqr;
        double filtered_total = 0;
        size_t filtered_count = 0;
        for (double value : samples) {
            if (value >= lower && value <= upper) {
                filtered_total += value;
                ++filtered_count;
            }
        }
        s.outlier_count = samples.size() - filtered_count;
        if (filtered_count > 0) s.outlier_filtered_avg_ms = filtered_total / filtered_count;
    }
    return s;
}

static void write_latency_json(std::ostream &out, const LatencyStats &s) {
    out << "{\"samples\":" << s.samples << ",\"failures\":" << s.failures
        << ",\"total_ms\":" << s.total_ms << ",\"avg_ms\":" << s.avg_ms
        << ",\"min_ms\":" << s.min_ms << ",\"p50_ms\":" << s.p50_ms
        << ",\"p95_ms\":" << s.p95_ms << ",\"p99_ms\":" << s.p99_ms
        << ",\"max_ms\":" << s.max_ms << ",\"ops_per_sec\":" << s.ops_per_sec
        << ",\"tags_per_sec\":" << s.tags_per_sec
        << ",\"outlier_method\":\"Tukey 1.5*IQR\",\"outlier_count\":" << s.outlier_count
        << ",\"outlier_filtered_avg_ms\":" << s.outlier_filtered_avg_ms << "}";
}

static size_t count_successes(const char *response) {
    const std::string text(response ? response : "");
    size_t count = 0, position = 0;
    while ((position = text.find("\"success\":true", position)) != std::string::npos) {
        ++count;
        position += 14;
    }
    return count;
}

static std::string make_terminal_batch_payload(const std::vector<const Tag *> &tags) {
    std::ostringstream payload;
    payload << '[';
    for (size_t index = 0; index < tags.size(); ++index) {
        if (index > 0) payload << ',';
        payload << "{\"tag_name\":\"" << tags[index]->name
                << "\",\"is_write\":true,\"value_type\":\"DINT\",\"value\":999999}";
    }
    payload << ']';
    return payload.str();
}

int main(int argc, char **argv) {
    std::string address = std::getenv("TEST_PLC_ADDRESS") ? std::getenv("TEST_PLC_ADDRESS")
                                                           : "192.168.0.1:44818";
    int slot = std::getenv("TEST_PLC_SLOT") ? std::atoi(std::getenv("TEST_PLC_SLOT")) : 0;
    std::string manifest_path = "examples/full_coverage_tags.json";
    std::string out_dir = "examples/full_coverage_results";
    bool skip_preflight = false;
    int benchmark_passes = 0;
    bool batch_benchmark = false;
    int batch_min_samples = 1000;
    double batch_min_seconds = 30.0;
    bool allow_writes = false;
    for (int i = 1; i < argc; ++i) {
        std::string a = argv[i];
        auto next = [&]() { return i + 1 < argc ? std::string(argv[++i]) : std::string(); };
        if (a == "--plc-address") address = next();
        else if (a == "--plc-slot") slot = std::atoi(next().c_str());
        else if (a == "--manifest") manifest_path = next();
        else if (a == "--out-dir") out_dir = next();
        else if (a == "--skip-preflight") skip_preflight = true;
        else if (a == "--benchmark-passes") benchmark_passes = std::atoi(next().c_str());
        else if (a == "--allow-writes") allow_writes = true;
        else if (a == "--batch-benchmark") batch_benchmark = true;
        else if (a == "--batch-min-tag-operations") batch_min_samples = std::atoi(next().c_str());
        else if (a == "--batch-min-seconds") batch_min_seconds = std::atof(next().c_str());
    }

    std::ifstream in(manifest_path);
    if (!in) {
        std::fprintf(stderr, "manifest-error: cannot open %s\n", manifest_path.c_str());
        return 2;
    }
    std::stringstream ss;
    ss << in.rdbuf();
    Json manifest = JsonParser(ss.str()).parse();
    const Json *cats = manifest.find("categories");
    if (!cats || cats->type != Json::Arr) {
        std::fprintf(stderr, "manifest-error: no categories array\n");
        return 2;
    }
    std::vector<Tag> tags;
    for (const auto &c : cats->arr) expand(c, tags);

    long writeable = 0, blocked = 0, readonly = 0;
    for (const auto &t : tags) {
        if (is_writeable(t.write)) ++writeable;
        else if (is_blocked(t.write)) ++blocked;
        else ++readonly;
    }
    std::printf("C/C++ full-coverage — %zu tags (writeable %ld, blocked %ld, read-only %ld)\n",
                tags.size(), writeable, blocked, readonly);
    if (benchmark_passes < 0 || (benchmark_passes > 0 && skip_preflight)) {
        std::fprintf(stderr, "benchmark passes must be non-negative and require preflight\n");
        return 2;
    }
    if (batch_min_samples <= 0 || batch_min_seconds < 0 ||
        (batch_benchmark && skip_preflight) || (batch_benchmark && benchmark_passes > 0)) {
        std::fprintf(stderr, "invalid or conflicting batch benchmark options\n");
        return 2;
    }
    if (benchmark_passes > 0 && !allow_writes) {
        std::fprintf(stderr, "benchmark mode writes terminal values; rerun with --allow-writes\n");
        return 2;
    }
    if (batch_benchmark && !allow_writes) {
        std::fprintf(stderr, "batch benchmark writes terminal DINT values; rerun with --allow-writes\n");
        return 2;
    }

    const uint8_t slots[1] = {(uint8_t)slot};
    int cid = eip_connect_with_route(address.c_str(), slots, 1, nullptr, 0, nullptr, 0);
    if (cid <= 0) {
        std::fprintf(stderr, "connect failed to %s slot %d\n", address.c_str(), slot);
        return 2;
    }

    std::map<std::string, CatStats> stats;

    // Phase 0 — preflight
    if (!skip_preflight) {
        long ok = 0, fail = 0;
        for (const auto &t : tags) {
            if (read_ok(cid, t)) ++ok;
            else {
                ++fail;
                std::fprintf(stderr, "setup-error: tag %s failed preflight\n", t.name.c_str());
            }
        }
        std::printf("Phase 0 — preflight  %ld/%ld\n", ok, ok + fail);
        if (fail > 0) { eip_disconnect(cid); return 2; }
    }

    if (batch_benchmark) {
        using Clock = std::chrono::steady_clock;
        const std::vector<size_t> sizes = {1, 5, 10, 20, 50, 100};
        std::vector<const Tag *> pool;
        for (const auto &tag : tags) {
            if (tag.category == "ctrl.DINT_array" && tag.kind == Kind::Dint && is_writeable(tag.write))
                pool.push_back(&tag);
            if (pool.size() == 100) break;
        }
        if (pool.size() != 100) {
            std::fprintf(stderr, "batch benchmark requires 100 controller DINT array tags, found %zu\n", pool.size());
            eip_disconnect(cid);
            return 2;
        }
        struct BatchRow { size_t size; LatencyStats reads, writes; };
        std::vector<BatchRow> rows;
        long total_failures = 0;
        std::printf("Batch benchmark — min %d tag operations and %.1fs per size/direction\n",
                    batch_min_samples, batch_min_seconds);
        for (size_t size : sizes) {
            const int required_batches = (batch_min_samples + (int)size - 1) / (int)size;
            std::vector<const Tag *> selected(pool.begin(), pool.begin() + (long)size);
            std::vector<const char *> names;
            for (const Tag *tag : selected) names.push_back(tag->name.c_str());
            const std::string write_payload = make_terminal_batch_payload(selected);
            std::vector<char> response(131072);
            auto read_call = [&]() {
                response[0] = '\0';
                const int rc = eip_read_tags_batch(cid, names.data(), (int)size,
                                                   response.data(), (int)response.size());
                return rc == 0 && count_successes(response.data()) == size;
            };
            auto write_call = [&]() {
                response[0] = '\0';
                const int rc = eip_execute_batch(cid, write_payload.c_str(), (int)size,
                                                 response.data(), (int)response.size());
                return rc == 0 && count_successes(response.data()) == size;
            };
            for (int warmup = 0; warmup < 10; ++warmup) {
                if (!read_call() || !write_call()) {
                    std::fprintf(stderr, "batch warm-up failed at size %zu\n", size);
                    eip_disconnect(cid);
                    return 2;
                }
            }

            std::vector<double> read_samples;
            long read_failures = 0;
            auto window = Clock::now();
            while ((int)(read_samples.size() + (size_t)read_failures) < required_batches ||
                   std::chrono::duration<double>(Clock::now() - window).count() < batch_min_seconds) {
                auto started = Clock::now();
                const bool ok = read_call();
                const double ms = std::chrono::duration<double, std::milli>(Clock::now() - started).count();
                if (ok) read_samples.push_back(ms); else ++read_failures;
            }

            std::vector<double> write_samples;
            long write_failures = 0;
            window = Clock::now();
            while ((int)(write_samples.size() + (size_t)write_failures) < required_batches ||
                   std::chrono::duration<double>(Clock::now() - window).count() < batch_min_seconds) {
                auto started = Clock::now();
                const bool ok = write_call();
                const double ms = std::chrono::duration<double, std::milli>(Clock::now() - started).count();
                if (ok) write_samples.push_back(ms); else ++write_failures;
            }

            LatencyStats reads = summarize(std::move(read_samples), read_failures, size);
            LatencyStats writes = summarize(std::move(write_samples), write_failures, size);
            std::printf("  size %3zu: read avg=%7.3fms filtered=%7.3fms; write avg=%7.3fms filtered=%7.3fms\n",
                        size, reads.avg_ms, reads.outlier_filtered_avg_ms,
                        writes.avg_ms, writes.outlier_filtered_avg_ms);
            rows.push_back({size, reads, writes});
            total_failures += read_failures + write_failures;
        }

        long terminal_verify_failures = 0;
        for (const Tag *tag : pool) {
            int value = 0;
            if (eip_read_dint(cid, tag->name.c_str(), &value) != 0 || value != 999999)
                ++terminal_verify_failures;
        }

        std::filesystem::create_directories(out_dir);
        std::time_t now = std::time(nullptr);
        char fname[512];
        std::snprintf(fname, sizeof(fname), "%s/cpp_batch_benchmark_%lld.json", out_dir.c_str(), (long long)now);
        std::ofstream out(fname);
        out << std::setprecision(10)
            << "{\n  \"schema_version\":1,\n  \"workload\":\"controller_dint_logical_batch_sizes\",\n"
            << "  \"binding\":\"cpp\",\n  \"binding_version\":\"" << eip_library_version() << "\",\n"
            << "  \"plc_address\":\"" << address << "\",\n  \"plc_slot\":" << slot
            << ",\n  \"batch_sizes\":[1,5,10,20,50,100],\n"
            << "  \"min_tag_operations_per_size_direction\":" << batch_min_samples
            << ",\n  \"min_seconds_per_size_direction\":" << batch_min_seconds
            << ",\n  \"read_api\":\"native C ABI batch read\",\n"
            << "  \"write_api\":\"native C ABI execute batch\",\n"
            << "  \"packet_policy\":\"default: max 20 operations and 504 bytes per CIP packet\",\n"
            << "  \"rows\":[\n";
        for (size_t index = 0; index < rows.size(); ++index) {
            if (index > 0) out << ",\n";
            out << "    {\"batch_size\":" << rows[index].size << ",\"reads\":";
            write_latency_json(out, rows[index].reads);
            out << ",\"writes\":";
            write_latency_json(out, rows[index].writes);
            out << '}';
        }
        out << "\n  ],\n  \"terminal_verify\":{\"ok\":" << (100 - terminal_verify_failures)
            << ",\"fail\":" << terminal_verify_failures << "},\n  \"result\":\""
            << (total_failures == 0 && terminal_verify_failures == 0 ? "PASS" : "FAIL") << "\"\n}\n";
        out.close();
        std::printf("wrote %s\n", fname);
        eip_disconnect(cid);
        return total_failures == 0 && terminal_verify_failures == 0 ? 0 : 1;
    }

    if (benchmark_passes > 0) {
        using Clock = std::chrono::steady_clock;
        std::vector<double> read_samples, write_samples;
        read_samples.reserve(tags.size() * (size_t)benchmark_passes);
        write_samples.reserve((size_t)writeable * (size_t)benchmark_passes);
        long read_failures = 0, write_failures = 0;
        std::printf("Benchmark — %d passes, %zu reads/pass, %ld writes/pass\n",
                    benchmark_passes, tags.size(), writeable);
        for (int pass = 0; pass < benchmark_passes; ++pass) {
            auto pass_start = Clock::now();
            for (const auto &t : tags) {
                auto started = Clock::now();
                bool ok = read_ok(cid, t);
                double ms = std::chrono::duration<double, std::milli>(Clock::now() - started).count();
                if (ok) read_samples.push_back(ms); else ++read_failures;
            }
            std::printf("  read pass %d/%d: %.1fs\n", pass + 1, benchmark_passes,
                        std::chrono::duration<double>(Clock::now() - pass_start).count());
        }
        for (int pass = 0; pass < benchmark_passes; ++pass) {
            auto pass_start = Clock::now();
            for (const auto &t : tags) {
                if (!is_writeable(t.write)) continue;
                auto started = Clock::now();
                int rc = write_terminal(cid, t);
                double ms = std::chrono::duration<double, std::milli>(Clock::now() - started).count();
                if (rc == 0) write_samples.push_back(ms); else ++write_failures;
            }
            std::printf("  write pass %d/%d: %.1fs\n", pass + 1, benchmark_passes,
                        std::chrono::duration<double>(Clock::now() - pass_start).count());
        }
        LatencyStats reads = summarize(std::move(read_samples), read_failures);
        LatencyStats writes = summarize(std::move(write_samples), write_failures);
        std::time_t now = std::time(nullptr);
        char fname[512];
        std::snprintf(fname, sizeof(fname), "%s/cpp_benchmark_%lld.json", out_dir.c_str(), (long long)now);
        std::ofstream out(fname);
        out << std::setprecision(10);
        out << "{\n  \"schema_version\":1,\n  \"workload\":\"full_coverage_manifest_sequential\",\n"
            << "  \"binding\":\"cpp\",\n  \"binding_version\":\"" << eip_library_version() << "\",\n"
            << "  \"plc_address\":\"" << address << "\",\n  \"plc_slot\":" << slot
            << ",\n  \"passes\":" << benchmark_passes << ",\n  \"tag_count\":" << tags.size()
            << ",\n  \"writeable_tag_count\":" << writeable
            << ",\n  \"warmup\":\"one full read-only preflight pass\",\n  \"reads\":";
        write_latency_json(out, reads);
        out << ",\n  \"writes\":";
        write_latency_json(out, writes);
        out << ",\n  \"result\":\"" << ((read_failures == 0 && write_failures == 0) ? "PASS" : "FAIL") << "\"\n}\n";
        out.close();
        std::printf("read avg/min/p95/p99/max %.3f/%.3f/%.3f/%.3f/%.3f ms\n",
                    reads.avg_ms, reads.min_ms, reads.p95_ms, reads.p99_ms, reads.max_ms);
        std::printf("write avg/min/p95/p99/max %.3f/%.3f/%.3f/%.3f/%.3f ms\n",
                    writes.avg_ms, writes.min_ms, writes.p95_ms, writes.p99_ms, writes.max_ms);
        std::printf("wrote %s\n", fname);
        eip_disconnect(cid);
        return read_failures == 0 && write_failures == 0 ? 0 : 1;
    }

    // Phase 1 — read every tag
    for (const auto &t : tags) {
        CatStats &s = stats[t.category];
        if (read_ok(cid, t)) ++s.read_ok;
        else ++s.read_fail;
    }
    std::printf("Phase 1 — read every tag: done\n");

    // Phase 2 — write random values to every writeable tag (STRINGs included)
    std::vector<Written> written;
    uint32_t rng = 0x1234567u;
    auto nextr = [&]() { rng = rng * 1664525u + 1013904223u; return rng; };
    for (size_t idx = 0; idx < tags.size(); ++idx) {
        const Tag &t = tags[idx];
        if (!is_writeable(t.write)) continue;
        CatStats &s = stats[t.category];
        int rc = -1;
        Written w{idx, t.kind, 0, 0.0, {}};
        switch (t.kind) {
            case Kind::Dint: { w.i = (int)(nextr() % 100000); rc = eip_write_dint(cid, t.name.c_str(), w.i); break; }
            case Kind::Int:  { w.i = (int)(nextr() % 20000) - 10000; rc = eip_write_int(cid, t.name.c_str(), (int16_t)w.i); break; }
            case Kind::Bool: { w.i = (int)(nextr() & 1); rc = eip_write_bool(cid, t.name.c_str(), w.i); break; }
            case Kind::Real: { w.r = (double)(nextr() % 1000); rc = eip_write_real(cid, t.name.c_str(), w.r); break; }
            case Kind::String: { w.s = "CPPCOV" + std::to_string(idx); rc = eip_write_string(cid, t.name.c_str(), w.s.c_str()); break; }
            case Kind::Udt: continue;
        }
        if (rc == 0) { ++s.write_ok; written.push_back(w); }
        else ++s.write_fail;
    }
    std::printf("Phase 2 — write writeable: done (%zu written)\n", written.size());

    // Phase 3 — verify writes via typed read-back
    for (const auto &w : written) {
        const Tag &t = tags[w.idx];
        CatStats &s = stats[t.category];
        bool ok = false;
        switch (w.kind) {
            case Kind::Dint: { int v = 0; ok = eip_read_dint(cid, t.name.c_str(), &v) == 0 && v == w.i; break; }
            case Kind::Int:  { int16_t v = 0; ok = eip_read_int(cid, t.name.c_str(), &v) == 0 && v == (int16_t)w.i; break; }
            case Kind::Bool: { int v = 0; ok = eip_read_bool(cid, t.name.c_str(), &v) == 0 && (v != 0) == (w.i != 0); break; }
            case Kind::Real: { double v = 0; ok = eip_read_real(cid, t.name.c_str(), &v) == 0 && std::fabs(v - w.r) < 0.001; break; }
            case Kind::String: { char b[512]; ok = eip_read_string(cid, t.name.c_str(), b, (int)sizeof(b)) == 0 && w.s == b; break; }
            case Kind::Udt: break;
        }
        if (ok) ++s.verify_ok; else ++s.verify_fail;
    }
    std::printf("Phase 3 — verify: done\n");

// Phase 4 — confirm any expected-blocked writes are still rejected
    for (const auto &t : tags) {
        if (!is_blocked(t.write)) continue;
        CatStats &s = stats[t.category];
        int rc = eip_write_string(cid, t.name.c_str(), "x");
        if (rc != 0) ++s.blocked_ok; else ++s.blocked_unexpected;
    }
    std::printf("Phase 4 — blocked-probe: done\n");

    // Phase 5 — settle writeable tags to a terminal state
    for (size_t idx = 0; idx < tags.size(); ++idx) {
        const Tag &t = tags[idx];
        if (!is_writeable(t.write)) continue;
        switch (t.kind) {
            case Kind::Dint: eip_write_dint(cid, t.name.c_str(), 999999); break;
            case Kind::Int: eip_write_int(cid, t.name.c_str(), (int16_t)9999); break;
            case Kind::Bool: eip_write_bool(cid, t.name.c_str(), 1); break;
            case Kind::Real: eip_write_real(cid, t.name.c_str(), 99.99); break;
            case Kind::String: eip_write_string(cid, t.name.c_str(), "SETTLED"); break;
            case Kind::Udt: break;
        }
    }
    std::printf("Phase 5 — settle: done\n");

    // Phase 6 — settle-verify a sample (first writeable tag per category)
    long settle_verify_ok = 0, settle_verify_fail = 0;
    std::map<std::string, bool> sampled;
    for (size_t idx = 0; idx < tags.size(); ++idx) {
        const Tag &t = tags[idx];
        if (!is_writeable(t.write) || sampled[t.category]) continue;
        sampled[t.category] = true;
        bool ok = false;
        switch (t.kind) {
            case Kind::Dint: { int v = 0; ok = eip_read_dint(cid, t.name.c_str(), &v) == 0 && v == 999999; break; }
            case Kind::Int: { int16_t v = 0; ok = eip_read_int(cid, t.name.c_str(), &v) == 0 && v == (int16_t)9999; break; }
            case Kind::Bool: { int v = 0; ok = eip_read_bool(cid, t.name.c_str(), &v) == 0 && v != 0; break; }
            case Kind::Real: { double v = 0; ok = eip_read_real(cid, t.name.c_str(), &v) == 0 && std::fabs(v - 99.99) < 0.001; break; }
            case Kind::String: { char b[512]; ok = eip_read_string(cid, t.name.c_str(), b, (int)sizeof(b)) == 0 && std::strcmp(b, "SETTLED") == 0; break; }
            case Kind::Udt: break;
        }
        if (ok) ++settle_verify_ok; else ++settle_verify_fail;
    }
    std::printf("Phase 6 — settle-verify: %ld/%ld\n", settle_verify_ok, settle_verify_ok + settle_verify_fail);

    eip_disconnect(cid);

    // Totals + report
    CatStats tot;
    std::printf("\nPer-category results:\n");
    std::printf("  %-32s %8s %8s %8s %8s %8s %8s\n", "category", "read+", "read-", "write+",
                "write-", "verify+", "blocked+");
    for (const auto &kv : stats) {
        const CatStats &s = kv.second;
        std::printf("  %-32s %8ld %8ld %8ld %8ld %8ld %8ld\n", kv.first.c_str(), s.read_ok,
                    s.read_fail, s.write_ok, s.write_fail, s.verify_ok, s.blocked_ok);
        tot.read_ok += s.read_ok; tot.read_fail += s.read_fail;
        tot.write_ok += s.write_ok; tot.write_fail += s.write_fail;
        tot.verify_ok += s.verify_ok; tot.verify_fail += s.verify_fail;
        tot.blocked_ok += s.blocked_ok; tot.blocked_unexpected += s.blocked_unexpected;
    }
    std::printf("  %-32s %8ld %8ld %8ld %8ld %8ld %8ld\n", "TOTAL", tot.read_ok, tot.read_fail,
                tot.write_ok, tot.write_fail, tot.verify_ok, tot.blocked_ok);

    long anomalies = tot.read_fail + tot.write_fail + tot.verify_fail + tot.blocked_unexpected +
                     settle_verify_fail;
    std::printf("\nSummary: reads=%ld/%zu  writes=%ld/%ld  verify=%ld/%ld  "
                "blocked_as_expected=%ld  unexpected_anomalies=%ld\n",
                tot.read_ok, tags.size(), tot.write_ok, tot.write_ok + tot.write_fail,
                tot.verify_ok, tot.verify_ok + tot.verify_fail, tot.blocked_ok, anomalies);
    bool pass = anomalies == 0 && tot.read_ok == (long)tags.size();
    std::printf("binding=cpp tags=%zu reads=%ld/%zu writes=%ld/%ld verify=%ld/%ld blocked=%ld "
                "anomalies=%ld RESULT=%s\n",
                tags.size(), tot.read_ok, tags.size(), tot.write_ok, tot.write_ok + tot.write_fail,
                tot.verify_ok, tot.verify_ok + tot.verify_fail, tot.blocked_ok, anomalies,
                pass ? "PASS" : "FAIL");

    // JSON result artifact
    std::time_t now = std::time(nullptr);
    char fname[512];
    std::snprintf(fname, sizeof(fname), "%s/cpp_%lld.json", out_dir.c_str(), (long long)now);
    std::ofstream out(fname);
    if (out) {
        out << "{\n  \"binding\": \"cpp\",\n  \"tags\": " << tags.size()
            << ",\n  \"reads\": " << tot.read_ok << ",\n  \"writes\": " << tot.write_ok
            << ",\n  \"verify\": " << tot.verify_ok << ",\n  \"blocked_as_expected\": "
            << tot.blocked_ok << ",\n  \"unexpected_anomalies\": " << anomalies
            << ",\n  \"result\": \"" << (pass ? "PASS" : "FAIL") << "\"\n}\n";
        std::printf("wrote %s\n", fname);
    }
    std::printf("RESULT: %s\n", pass ? "PASS" : "FAIL");
    return pass ? 0 : 1;
}
